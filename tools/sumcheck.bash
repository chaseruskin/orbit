if cat orbit-$1-checksums.txt | grep $(sha256sum orbit-$1-$2.zip) then
    exit 0
else
    exit 1
fi