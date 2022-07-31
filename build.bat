SET CURRENT_DIR=%~dp0%

docker build . -t ofv/windows -f %CURRENT_DIR%docker\Dockerfile.windows
docker run --rm -it -v %CURRENT_DIR%:/app ofv/windows

pause
