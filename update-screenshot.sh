#!/bin/bash
rm demo*.png
NO_TEXT=Y cargo run &
PID=$!
sleep 1
scrot -w $(xdo id -p $PID) 'demo.png' 
kill $PID
convert demo.png -transparent black -fill black -opaque white -fuzz 15% -trim +repage demo.png
