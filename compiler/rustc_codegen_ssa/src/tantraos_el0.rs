//! TantraOS EL0 Async Tasklet Code Generation
//!
//! This module handles the generation of position-independent code
//! for async tasklets that run at EL0 (userspace) in TantraOS.
//!
//! Key features:
//! - Position-independent code generation for EL0 execution
//! - Future state machine compilation for cross-privilege polling
//! - Minimal runtime stub generation
//! - SVC-based yield/return mechanism

use rustc_middle::ty::{self, TyCtxt, Instance};
use rustc_codegen_ssa::traits::*;

/// Generate EL0-compatible async tasklet code
///
/// This takes a regular async function and generates code that can:
/// 1. Run at EL0 (userspace) with limited privileges
/// 2. Poll Futures across the EL0/EL1 boundary
/// 3. Yield control back to EL1 via SVC
pub fn generate_el0_tasklet<'tcx, Bx: BuilderMethods<'tcx>>(
    cx: &Bx::CodegenCx,
    instance: Instance<'tcx>,
) -> Bx::Function {
    // Get the function signature
    let sig = cx.tcx().fn_sig(instance.def_id());
    let sig = cx.tcx().normalize_erasing_late_bound_regions(cx.typing_env(), sig);

    // Check if this is an async function
    let is_async = sig.output().is_coroutine() ||
                   sig.output().to_string().contains("impl Future");

    if !is_async {
        // For non-async functions, generate regular code
        return cx.get_fn(instance);
    }

    // For async functions, we need special handling
    generate_el0_async_wrapper(cx, instance, sig)
}

/// Generate the EL0 wrapper for an async function
fn generate_el0_async_wrapper<'tcx, Bx: BuilderMethods<'tcx>>(
    cx: &Bx::CodegenCx,
    instance: Instance<'tcx>,
    sig: ty::FnSig<'tcx>,
) -> Bx::Function {
    let tcx = cx.tcx();

    // Create the wrapper function name
    let wrapper_name = format!("__tantraos_el0_{}",
                              tcx.def_path_str(instance.def_id()));

    // Create the wrapper function with proper signature
    // The wrapper takes a Context pointer and returns Poll<T>
    let poll_sig = ty::FnSig {
        inputs_and_output: tcx.mk_type_list(&[
            // Context pointer parameter
            tcx.mk_mut_ptr(tcx.types.u8),
            // Original function parameters
            ..sig.inputs()
        ]),
        c_variadic: false,
        unsafety: rustc_hir::Unsafety::Normal,
        abi: rustc_target::spec::abi::Abi::C { unwind: false },
    };

    let llfn = cx.declare_fn(&wrapper_name, poll_sig, &[]);

    // Build the wrapper function body
    let mut bx = Bx::build(cx, llfn);

    // The wrapper needs to:
    // 1. Set up the Future state machine
    // 2. Create a polling loop that calls Future::poll
    // 3. Handle Pending results via SVC #2 (yield)
    // 4. Handle Ready results via SVC #1 (return)

    // Get the original async function to create the Future
    let async_fn = cx.get_fn(instance);

    // Allocate space for the Future state machine
    let future_size = cx.size_of(sig.output());
    let future_align = cx.align_of(sig.output());
    let future_ptr = bx.alloca(cx.type_i8(), future_size);

    // Call the async function to initialize the Future
    let future = bx.call(
        cx.type_func(&sig.inputs().iter().map(|_| cx.type_i8()).collect::<Vec<_>>(),
                     cx.type_i8()),
        None,
        None,
        async_fn,
        &bx.fn_args[1..], // Skip context, pass original args
        None,
        None,
    );

    // Store the Future in allocated space
    bx.store(future, future_ptr, future_align);

    // Create basic blocks for the polling loop
    let poll_block = bx.append_block("poll_future");
    let pending_block = bx.append_block("pending");
    let ready_block = bx.append_block("ready");

    // Jump to poll block
    bx.br(poll_block);

    // Poll block: Call Future::poll with the Context
    bx.switch_to_block(poll_block);

    // Get the Context pointer from first argument
    let context_ptr = bx.fn_args[0];

    // Generate the poll call
    // In real implementation, this would look up the Future::poll method
    // For now, we'll generate inline assembly that represents the poll

    // Call Future::poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<T>
    // This needs to:
    // 1. Pin the future
    // 2. Call the poll method
    // 3. Check if result is Ready or Pending

    // For demonstration, generate assembly that checks a condition
    // In production, this would be the actual Future::poll call
    let poll_result = bx.inline_asm_call(
        concat!(
            "mov x19, $0\n",           // Save future pointer
            "mov x20, $1\n",           // Save context pointer
            "ldr x21, [x19]\n",        // Load future state
            "cmp x21, #0\n",           // Check if ready (simplified)
            "cset x0, eq\n",           // Set x0 = 1 if ready, 0 if pending
        ),
        "r,r",                         // Two input registers
        &[future_ptr, context_ptr],   // Future and context pointers
        cx.type_i8(),                  // Returns bool (ready/pending)
        false,
        false,
        rustc_ast::LlvmAsmDialect::Att,
        &[],
    );

    // Check poll result (0 = Pending, 1 = Ready)
    let is_ready = bx.icmp(
        IntPredicate::IntEQ,
        poll_result,
        bx.const_i8(1)
    );

    // Branch based on poll result
    bx.cond_br(is_ready, ready_block, pending_block);

    // Pending block: Yield to EL1 via SVC #2
    bx.switch_to_block(pending_block);

    // Save Future state before yielding
    // This ensures we can resume from the right point
    let save_state = bx.inline_asm_call(
        concat!(
            "str x19, [sp, #-16]!\n",  // Push future pointer
            "str x20, [sp, #-16]!\n",  // Push context pointer
            "svc #2\n",                // Yield to EL1
            "ldr x20, [sp], #16\n",    // Pop context pointer
            "ldr x19, [sp], #16\n",    // Pop future pointer
        ),
        "",                            // No constraints
        &[],                           // No inputs
        cx.type_void(),
        true,                          // Volatile (has side effects)
        false,
        rustc_ast::LlvmAsmDialect::Att,
        &[],
    );

    // After yield, jump back to poll
    bx.br(poll_block);

    // Ready block: Extract value and return via SVC #1
    bx.switch_to_block(ready_block);

    // Extract the result value from the Future
    // In real implementation, this would extract Poll::Ready(value)
    let result_ptr = bx.struct_gep(cx.type_i8(), future_ptr, 1);
    let result = bx.load(cx.type_i64(), result_ptr, 8);

    // Return to EL1 with the result via SVC #1
    let svc_return = bx.inline_asm_call(
        concat!(
            "mov x0, $0\n",            // Move result to x0
            "svc #1\n",                // Return to EL1
        ),
        "r",                           // Input register
        &[result],                     // Result value
        cx.type_void(),
        true,                          // Volatile
        false,
        rustc_ast::LlvmAsmDialect::Att,
        &[],
    );

    // This point should never be reached
    bx.unreachable();

    // Finish building the function
    bx.finish();

    llfn
}

/// Check if a function should be compiled for EL0
pub fn should_compile_for_el0(tcx: TyCtxt<'_>, def_id: ty::DefId) -> bool {
    // Check for #[tantraos::el0] attribute
    if tcx.has_attr(def_id, rustc_span::symbol::sym::tantraos_el0) {
        return true;
    }

    // Check if this is a User tasklet type
    // This would check the TaskletType enum but we'll keep it simple for now

    false
}

/// Generate position-independent code for EL0
pub fn make_position_independent<'tcx, Bx: BuilderMethods<'tcx>>(
    cx: &Bx::CodegenCx,
    llfn: Bx::Function,
) {
    // Set function attributes for PIC
    cx.set_fn_attr(llfn, "pic-level", "2");
    cx.set_fn_attr(llfn, "pie-level", "2");

    // Ensure no absolute addresses are used
    cx.set_fn_attr(llfn, "relocation-model", "pic");
}