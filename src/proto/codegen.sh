protoc --rust_out=gen *.proto
cd format 
protoc --rust_out=../gen/format *.proto