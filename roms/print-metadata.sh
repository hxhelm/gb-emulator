printf "%-60s %-10s %-10s %-10s\n" "ROM Path" "MBC" "ROM Size" "RAM Size"
find . -type f -name "*.gb" | sed 's|^\./||' | while read -r f; do
  read -r mbc rom ram <<< $(xxd -ps -s 0x0147 -l 3 "$f" | sed 's/\(..\)/0x\1 /g')
  printf "%-60s %-10s %-10s %-10s\n" "$f" "$mbc" "$rom" "$ram"
done

