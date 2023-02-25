# batch update metadata
# creates an array of update msgs that are sent in a single tx
# needs csv file with token_id, token_uri

KEY=$(starsd keys show $ADMIN | jq -r .name)
CSV_FILE="metadata.csv"

# Stores the token IDs and URIs
tokens=()

# Read the CSV file line by line
while IFS=',' read -r token_id token_uri || [[ -n "$token_id" ]]; do
  # Push the values into the tokens array
  tokens+=("$token_id" "$token_uri")
  echo $token_id $token_uri
done < "$CSV_FILE"

# Loop over the tokens array and print out the values
msgs=()
for (( i=0; i<${#tokens[@]}; i+=2 )); do
token_id=${tokens[i]}
token_uri=${tokens[i+1]}
echo "Token ID: $token_id"
echo "Token URI: $token_uri"

MSG=$(cat <<EOF
{
    "update_token_metadata": {
        "token_id": "$token_id",
        "token_uri": "$token_uri"
    }
}
EOF
)

echo $MSG
msgs+=("$MSG")
done

# Set the IFS variable to a comma character
IFS=','

# Convert the array to a string with the desired format
full_tx_msg=$(printf "[%s]" "${msgs[*]}")

# Print the full tx msg
echo "$full_tx_msg"

starsd tx wasm execute $SG721 "$MSG" \
--gas-prices 0.025ustars --gas auto --gas-adjustment 1.9 \
--from $KEY -y -b block -o json | jq .