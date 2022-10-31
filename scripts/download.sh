curl -s https://api.github.com/repos/public-awesome/launchpad/releases/latest \
| grep ".*wasm" \
| cut -d : -f 2,3 \
| tr -d \" \
| wget -qi -
