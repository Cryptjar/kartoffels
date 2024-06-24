#![cfg_attr(target_arch = "riscv64", no_std, no_main)]

kartoffels_vm_tests::test! {
    r#"
    .global _start
    .attribute arch, "rv64im"

    _start:
        li x1, -100
        li x2, 20
        div x3, x1, x2
        div x4, x2, x0
        ebreak
    "#
}

/*
 * x1 = -100
 * x2 = 20
 * x3 = -5
 * x4 = -1
 */
