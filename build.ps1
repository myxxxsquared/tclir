
$env:DEFMT_LOG = 'info'
cargo build --release
arm-none-eabi-objcopy -O binary .\target\thumbv7m-none-eabi\release\tclir .\target\thumbv7m-none-eabi\release\tclir.bin
stm32flash -w .\target\thumbv7m-none-eabi\release\tclir.bin COM5

