@echo off
rem Button-free flash for RP2040 on Windows.
rem
rem picotool on Windows rejects single-shot forced commands for RP2040
rem ("picotool load -f"), so this script uses the supported two-step flow:
rem reboot the running firmware into BOOTSEL via its vendor reset interface,
rem then load. Works from both application mode and BOOTSEL mode.

picotool reboot -f -u >nul 2>&1

for /l %%i in (1,1,10) do (
  picotool load -x -t elf %1 && exit /b 0
  ping -n 2 127.0.0.1 >nul
)
echo picotool load failed after 10 attempts 1>&2
exit /b 1
