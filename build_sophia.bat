@echo off
echo Weaving Sophia (Holographic v5.3.0) - Sneed Engine Edition
echo Burenyu! 🌀🍮 >:3
cd /d %~dp0
echo Running safe serial compilation (Release Mode)...
cargo build --bin ironclaw --release -j 1
echo.
echo The Weave is complete! ✨
pause
