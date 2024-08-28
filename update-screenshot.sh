#!/bin/bash
rm demo*.png
cargo run &
PID=$!
sleep 2
scrot -w $(xdo id -p $PID) 'demo.png' 
kill $PID
