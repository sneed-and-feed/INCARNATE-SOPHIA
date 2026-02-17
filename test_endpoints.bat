@echo off
echo Testing with testtoken123...
curl -v -H "Authorization: Bearer testtoken123" http://127.0.0.1:3000/api/health
echo.
curl -v -H "Authorization: Bearer testtoken123" http://127.0.0.1:3000/api/extensions
echo.

echo.
echo Testing with SophiaSecret123...
curl -v -H "Authorization: Bearer SophiaSecret123" http://127.0.0.1:3000/api/health
echo.
curl -v -H "Authorization: Bearer SophiaSecret123" http://127.0.0.1:3000/api/extensions
echo.
