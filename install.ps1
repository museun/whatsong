cargo build --release --bin whatsong_client
if ($?) {
    echo "copying client to stream directory"
    copy ./target/release/whatsong_client.exe c:/dev/stream/whatsong_client.exe
} 
