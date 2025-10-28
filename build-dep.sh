rm -rf ./deps
mkdir ./deps

cd ./deps
git clone --depth 1 https://github.com/raysan5/raylib.git raylib
cd ./raylib
mkdir ./build
cd ./build
cmake -DBUILD_SHARED_LIBS=OFF -DCMAKE_BUILD_TYPE=Release ..
make

cd ../../
git clone --depth 1 https://github.com/glfw/glfw.git glfw
cd ./glfw
cmake -DBUILD_SHARED_LIBS=OFF -DCMAKE_BUILD_TYPE=Release .
make
