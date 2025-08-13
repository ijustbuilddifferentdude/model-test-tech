cargo run --release -- \
  --input ./data/interval_1.csv \
  --output ./data/out/interval.csv \
  --horizon-sec 1.0 \
  --eps-min 0.0003 \
  --alpha 0.5 \