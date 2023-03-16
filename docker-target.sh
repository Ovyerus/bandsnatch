# Script to remap Docker's `$TARGETARCH` to more appropriate values for rustup.
if [ "$TARGETARCH" = "amd64" ]; then
    echo "x86_64"
elif [ "$TARGETARCH" = "arm64" ]; then
    echo "aarch64"
else
    echo "$TARGETARCH"
fi