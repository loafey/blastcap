cargo build
cargo run &
cargo run &

for job in `jobs -p`
do
echo $job
    wait $job || let "FAIL+=1"
done
