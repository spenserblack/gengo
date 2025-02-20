@echo off
:start
cls
echo Simple Calculator
echo ================
echo 1. Addition
echo 2. Subtraction
echo 3. Multiplication
echo 4. Division
echo 5. Exit
echo.

set /p choice="Enter your choice (1-5): "
if %choice%==5 goto end

set /p num1="Enter first number: "
set /p num2="Enter second number: "

if %choice%==1 set /a result=%num1%+%num2%
if %choice%==2 set /a result=%num1%-%num2%
if %choice%==3 set /a result=%num1%*%num2%
if %choice%==4 set /a result=%num1%/%num2%

echo.
echo Result: %result%
echo.
pause
goto start

:end
echo Thanks for using the calculator!
pause