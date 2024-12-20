验证步骤：
1. 进入当前arceos根目录（`cd /path/to/arceos`）
2. `cd payload`
3. `python3 build.py dynamic`编译动态链接的ELF文件 or `python3 build.py static`静态编译
4. `cd ..`
5. `make run`
6. 预期输出
    ```plaintext
    arch = riscv64
    platform = riscv64-qemu-virt
    target = riscv64gc-unknown-none-elf
    smp = 1
    build_mode = release
    log_level = warn

    H
    [ABI:Exit] Exit!
    ```