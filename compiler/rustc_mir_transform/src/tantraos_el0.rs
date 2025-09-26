//! TantraOS EL0 Async Transformation Pass
//!
//! This MIR transformation pass modifies async functions marked for EL0
//! execution to be compatible with cross-privilege-level polling.

use rustc_middle::mir::*;
use rustc_middle::ty::{self, TyCtxt, Instance, GenericArgsRef};
use rustc_span::{DUMMY_SP, Symbol, sym};
use rustc_index::IndexVec;
use rustc_target::abi::VariantIdx;

/// Transform async functions for EL0 execution
pub struct TantraosEl0Transform;

impl<'tcx> MirPass<'tcx> for TantraosEl0Transform {
    fn run_pass(&self, tcx: TyCtxt<'tcx>, body: &mut Body<'tcx>) {
        // Only process if we're targeting TantraOS
        if tcx.sess.target.os != "tantraos" {
            return;
        }

        // Check if this function should run at EL0
        if !should_transform_for_el0(tcx, body.source.def_id()) {
            return;
        }

        // Transform the function
        transform_for_el0(tcx, body);
    }
}

/// Check if a function should be transformed for EL0
fn should_transform_for_el0(tcx: TyCtxt<'_>, def_id: DefId) -> bool {
    // Check for #[tantraos::el0] attribute
    if tcx.has_attr(def_id, Symbol::intern("tantraos_el0")) {
        return true;
    }

    // Check if this is marked as a User tasklet
    // Look for functions that are async and meant for userspace
    let attrs = tcx.get_attrs(def_id, Symbol::intern("tantraos"));
    if attrs.iter().any(|attr| {
        attr.name_value_literal_span().is_some() &&
        attr.value_str().map_or(false, |s| s.as_str() == "user")
    }) {
        return true;
    }

    // For now, also check function names for user/el0/uart patterns
    let name = tcx.def_path_str(def_id);
    name.contains("el0") || name.contains("user") || name.contains("uart")
}

/// Transform a function body for EL0 execution
fn transform_for_el0<'tcx>(tcx: TyCtxt<'tcx>, body: &mut Body<'tcx>) {
    // Check if this is an async function (has yield points)
    let is_async = body.basic_blocks.iter().any(|bb| {
        bb.terminator.as_ref().map_or(false, |t| {
            matches!(t.kind, TerminatorKind::Yield { .. })
        })
    });

    if is_async {
        // Transform async functions with proper state machine handling
        transform_async_for_el0(tcx, body);
    } else {
        // For regular functions, just add EL0 transition wrappers
        add_el0_wrappers(tcx, body);
    }
}

/// Transform async function for EL0 execution
fn transform_async_for_el0<'tcx>(tcx: TyCtxt<'tcx>, body: &mut Body<'tcx>) {
    // Transform yield points to use SVC for cross-privilege polling
    transform_yield_points(tcx, body);

    // Inject state preservation around yields
    inject_state_preservation(tcx, body);

    // Add return transformation for completion
    transform_returns_for_el0(tcx, body);
}

/// Add simple EL0 wrappers for non-async functions
fn add_el0_wrappers<'tcx>(tcx: TyCtxt<'tcx>, body: &mut Body<'tcx>) {
    // For non-async functions, we just need to handle returns
    transform_returns_for_el0(tcx, body);
}

/// Transform yield points to use SVC
fn transform_yield_points<'tcx>(tcx: TyCtxt<'tcx>, body: &mut Body<'tcx>) {
    let source_info = SourceInfo::outermost(DUMMY_SP);

    // Process each basic block
    for (bb_idx, block) in body.basic_blocks.as_mut().iter_enumerated_mut() {
        if let Some(ref mut terminator) = block.terminator {
            match &terminator.kind {
                TerminatorKind::Yield { value, resume, resume_arg, drop } => {
                    // Save the yield information
                    let yield_value = value.clone();
                    let resume_bb = *resume;
                    let resume_place = resume_arg.clone();
                    let drop_bb = *drop;

                    // Create inline assembly for SVC #2 (yield)
                    // This will yield to EL1 and wait for rescheduling
                    let svc_yield = Rvalue::InlineAsm(Box::new(InlineAsmOperand {
                        template: vec!["svc #2".to_string()],
                        operands: vec![],
                        options: InlineAsmOptions::NOSTACK,
                    }));

                    // Add statement to perform the SVC
                    block.statements.push(Statement {
                        source_info,
                        kind: StatementKind::Assign(Box::new((
                            Place::from(Local::new(0)), // Dummy place
                            svc_yield,
                        ))),
                    });

                    // Replace yield terminator with goto to resume
                    terminator.kind = TerminatorKind::Goto {
                        target: resume_bb,
                    };
                }
                _ => {}
            }
        }
    }
}

/// Inject state preservation around yield points
fn inject_state_preservation<'tcx>(tcx: TyCtxt<'tcx>, body: &mut Body<'tcx>) {
    let source_info = SourceInfo::outermost(DUMMY_SP);

    // For each yield point, we need to:
    // 1. Save local state to a stable location
    // 2. Restore state after resumption

    for (bb_idx, block) in body.basic_blocks.as_mut().iter_enumerated_mut() {
        // Check if this block resumes from a yield
        // In a real implementation, we'd track this from yield transformations

        // Add state restoration at the beginning of resume blocks
        if needs_state_restoration(bb_idx) {
            // Insert state restoration instructions at the beginning
            let restore_asm = Statement {
                source_info,
                kind: StatementKind::Assign(Box::new((
                    Place::from(Local::new(0)),
                    Rvalue::InlineAsm(Box::new(InlineAsmOperand {
                        template: vec![
                            "ldr x19, [sp], #16".to_string(),  // Restore saved registers
                            "ldr x20, [sp], #16".to_string(),
                        ],
                        operands: vec![],
                        options: InlineAsmOptions::NOSTACK,
                    })),
                ))),
            };

            block.statements.insert(0, restore_asm);
        }
    }
}

/// Transform return statements for EL0
fn transform_returns_for_el0<'tcx>(tcx: TyCtxt<'tcx>, body: &mut Body<'tcx>) {
    let source_info = SourceInfo::outermost(DUMMY_SP);

    for block in body.basic_blocks.as_mut() {
        if let Some(ref mut terminator) = block.terminator {
            if matches!(terminator.kind, TerminatorKind::Return) {
                // Add SVC #1 before return to signal completion to EL1
                let svc_return = Statement {
                    source_info,
                    kind: StatementKind::Assign(Box::new((
                        Place::from(Local::new(0)),
                        Rvalue::InlineAsm(Box::new(InlineAsmOperand {
                            template: vec!["svc #1".to_string()],
                            operands: vec![],
                            options: InlineAsmOptions::NOSTACK,
                        })),
                    ))),
                };

                // Insert SVC before the return
                block.statements.push(svc_return);
            }
        }
    }
}

/// Helper to check if a block needs state restoration
fn needs_state_restoration(bb: BasicBlock) -> bool {
    // In a complete implementation, this would track resume points
    // from yield transformations
    false
}

// Required structures for inline assembly
struct InlineAsmOperand {
    template: Vec<String>,
    operands: Vec<InlineAsmOperandInner>,
    options: InlineAsmOptions,
}

enum InlineAsmOperandInner {
    In { reg: String, value: Operand },
    Out { reg: String, place: Place },
}

bitflags::bitflags! {
    struct InlineAsmOptions: u32 {
        const NOSTACK = 0x1;
        const PURE = 0x2;
        const NOMEM = 0x4;
    }
}

use rustc_middle::mir::visit::MutVisitor;
use rustc_middle::mir::{DefId, Local, Place, Operand, Rvalue, Statement, StatementKind};