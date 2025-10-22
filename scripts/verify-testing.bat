@echo off
echo Verifying 66+ Testing Implementation...
echo ======================================

echo Checking required components...
set ALL_PRESENT=true

if exist ".github\workflows\testing-66plus.yml" (
    echo   [✓] .github/workflows/testing-66plus.yml
) else (
    echo   [ ] .github/workflows/testing-66plus.yml
    set ALL_PRESENT=false
)

if exist "docs\testing\66plus-testing-implementation.md" (
    echo   [✓] docs/testing/66plus-testing-implementation.md
) else (
    echo   [ ] docs/testing/66plus-testing-implementation.md
    set ALL_PRESENT=false
)

if exist "scripts\generate-test-files.py" (
    echo   [✓] scripts/generate-test-files.py
) else (
    echo   [ ] scripts/generate-test-files.py
    set ALL_PRESENT=false
)

if exist "scripts\setup-testing-framework.ps1" (
    echo   [✓] scripts/setup-testing-framework.ps1
) else (
    echo   [ ] scripts/setup-testing-framework.ps1
    set ALL_PRESENT=false
)

echo.
echo Checking test directories...

if exist "tests\happy_path" (
    echo   [✓] tests/happy_path
) else (
    echo   [ ] tests/happy_path
)

if exist "tests\boundary" (
    echo   [✓] tests/boundary
) else (
    echo   [ ] tests/boundary
)

if exist "tests\equivalence" (
    echo   [✓] tests/equivalence
) else (
    echo   [ ] tests/equivalence
)

if exist "tests\state" (
    echo   [✓] tests/state
) else (
    echo   [ ] tests/state
)

if exist "tests\api_contract" (
    echo   [✓] tests/api_contract
) else (
    echo   [ ] tests/api_contract
)

if exist "tests\auth" (
    echo   [✓] tests/auth
) else (
    echo   [ ] tests/auth
)

if exist "tests\secrets" (
    echo   [✓] tests/secrets
) else (
    echo   [ ] tests/secrets
)

if exist "tests\smoke" (
    echo   [✓] tests/smoke
) else (
    echo   [ ] tests/smoke
)

if exist "tests\regression" (
    echo   [✓] tests/regression
) else (
    echo   [ ] tests/regression
)

echo.
if "%ALL_PRESENT%"=="true" (
    echo ✅ Core components are in place!
    echo The 66+ testing framework has been successfully set up.
) else (
    echo ❌ Some components are missing.
)

echo.
echo Next steps:
echo 1. Run tests with: cargo test
echo 2. Check CI/CD execution: .github/workflows/testing-66plus.yml