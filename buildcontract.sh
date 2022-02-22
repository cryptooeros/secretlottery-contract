#!/bin/bash

#Build Flag
PARAM=$1
####################################    Constants    ##################################################

TXFLAG=" --gas auto --gas 4000000 --gas-prices=1.0uscrt"
WALLET="--from secworkshop"
WASMFILE="secret_lootbox.wasm"

FILE_UPLOADHASH="uploadtx.txt"
FILE_LOTTERY_CONTRACT_ADDR="contractaddr.txt"
FILE_CODE_ID="code.txt"

ADDR_SECWORKSHOP="secret179v8tkkhuyj6qg39v328csfevh7rx7j5udrvge"
ADDR_ACHILLES="secret154d0vg8m7khzmqh8nxf0nduen088v8st80q03t"
CONTRACT_VBLCK="juno1j5rl5sy40nmlqyugphgh5hnyrmj2cc5h7swy9x8rm0jkxy566nlqcx0jmv"


###################################################################################################
###################################################################################################
###################################################################################################
###################################################################################################
#Environment Functions
CreateEnv() {
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    source $HOME/.cargo/env

    rustup default stable
    rustup target list --installed
    rustup target add wasm32-unknown-unknown

    rustup install nightly
    rustup target add wasm32-unknown-unknown --toolchain nightly

    apt install build-essential

    cargo install cargo-generate --features vendored-openssl
}

InstallCli() {
    sudo wget -O /usr/bin/secretcli https://github.com/scrtlabs/SecretNetwork/releases/download/v1.2.2/secretcli-Linux
    sudo chmod a+x /usr/bin/secretcli
    mkdir -p ~/.secretd/config
    
    sudo echo 'chain-id = "pulsar-2"
    keyring-backend = "test"
    node = "tcp://testnet.securesecrets.org:26657"
    output = "text"
    broadcast-node = "sync"' > ~/.secretd/config/config.toml

}

#Build Optimized Contracts
OptimizeBuild() {

    echo "================================================="
    echo "Optimize Build Start"
    
    sudo docker run --rm -v "$(pwd)":/contract \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  enigmampc/secret-contract-optimizer  
}

RustBuild() {

    echo "================================================="
    echo "Rust Optimize Build Start"

    RUSTFLAGS='-C link-arg=-s' cargo wasm

    mkdir artifacts
    cp target/wasm32-unknown-unknown/release/$WASMFILE artifacts/$WASMFILE
}

#Writing to FILE_UPLOADHASH
Upload() {
    #secretcli tx compute store artifacts/$WASMFILE $WALLET $TXFLAG
    echo "================================================="
    echo "Upload $WASMFILE"
    UPLOADTX=$(secretcli tx compute store artifacts/$WASMFILE $WALLET $TXFLAG --output json -y | jq -r '.txhash')
    echo "Upload txHash:"$UPLOADTX
    
    #save to FILE_UPLOADHASH
    echo $UPLOADTX > $FILE_UPLOADHASH
    echo "wrote last transaction hash to $FILE_UPLOADHASH"
}

#Read code from FILE_UPLOADHASH
GetCode() {
    echo "================================================="
    echo "Get code from transaction hash written on $FILE_UPLOADHASH"
    
    #read from FILE_UPLOADHASH
    TXHASH=$(cat $FILE_UPLOADHASH)
    echo "read last transaction hash from $FILE_UPLOADHASH"
    echo $TXHASH
    
    # secretcli q tx $TXHASH 
    # QUERYTX="secretcli q tx $TXHASH --output json"
    #secretcli query tx $TXHASH 
	CODE_ID=$(secretcli q tx $TXHASH --output json | jq -r '.logs[0].events[-1].attributes[3].value')
	echo "Contract Code_id:"$CODE_ID

    #save to FILE_CODE_ID
    echo $CODE_ID > $FILE_CODE_ID
}

#Instantiate Contract
Instantiate() {
    echo "================================================="
    echo "Instantiate Contract"
    
    #read from FILE_CODE_ID
    CODE_ID=$(cat $FILE_CODE_ID)
    #INSTANTIATETX=$(secretcli tx compute instantiate $CODE_ID '{"name":"secret_lottery", "ticket_count":100, "golden": 97 }' --label "Lottery$CODE_ID" --amount 1SCRT $WALLET -y | jq -r '.txhash')
    #echo $INSTANTIATETX
    #secretcli query tx $INSTANTIATETX

    secretcli tx compute instantiate $CODE_ID '{"name":"secret_lottery", "ticket_count":100, "golden": 97 }' --label "Lottery$CODE_ID" --amount 1000000uscrt $WALLET -y
}

#Get Instantiated Contract Address
GetContractAddress() {
    echo "================================================="
    echo "Get contract address by code"
    
    #read from FILE_CODE_ID
    CODE_ID=$(cat $FILE_CODE_ID)
    echo $CODE_ID

    secretcli query compute list-contract-by-code $CODE_ID
    CONTRACT_ADDR=$(secretcli query compute list-contract-by-code $CODE_ID --output json | jq -r '.[0].address')
    
    echo "Contract Address : "$CONTRACT_ADDR

    #save to FILE_LOTTERY_CONTRACT_ADDR
    echo $CONTRACT_ADDR > $FILE_LOTTERY_CONTRACT_ADDR
}


###################################################################################################
###################################################################################################
###################################################################################################
###################################################################################################
#Send initial tokens
BuyTicket() {
    CONTRACT_LOTTERY=$(cat $FILE_LOTTERY_CONTRACT_ADDR)
    secretcli tx compute execute $CONTRACT_LOTTERY '{ "buy_ticket": { "ticket_id": 1 }}' $WALLET --amount 10000uscrt
    
}

EndLottery() {
    CONTRACT_LOTTERY=$(cat $FILE_LOTTERY_CONTRACT_ADDR)
    secretcli tx compute execute $CONTRACT_LOTTERY '{ "end_lottery": {} }' $WALLET
}

SetPrice() {
    CONTRACT_LOTTERY=$(cat $FILE_LOTTERY_CONTRACT_ADDR)
    junod tx wasm execute $CONTRACT_LOTTERY '{"set_price":{"denom":"ujuno", "price":"100"}}' $WALLET $TXFLAG
}

WithdrawAll() {
    CONTRACT_LOTTERY=$(cat $FILE_LOTTERY_CONTRACT_ADDR)
    junod tx wasm execute $CONTRACT_LOTTERY '{"withdraw_all":{}}' $WALLET $TXFLAG
}

PrintGetInfo() {
    CONTRACT_LOTTERY=$(cat $FILE_LOTTERY_CONTRACT_ADDR)
    junod query wasm contract-state smart $CONTRACT_LOTTERY '{"get_info":{}}' $NODECHAIN
}

PrintPoolContractState() {
    junod query wasm list-code $NODECHAIN --output json
    junod query wasm list-contract-by-code 16 $NODECHAIN
    
}

PrintWalletBalance() {
    junod query bank balances $ADDR_ADMIN $NODECHAIN
}

#################################### End of Function ###################################################
if [[ $PARAM == "" ]]; then
    RustBuild
    Upload
sleep 7
    GetCode
sleep 7
    Instantiate
sleep 7
    GetContractAddress
# sleep 5
   # SendInitialFund
# sleep 3
#     SetPrice
# sleep 5
#     PrintGetInfo
else
    $PARAM
fi

# OptimizeBuild
# Upload
# GetCode
# Instantiate
# GetContractAddress
# CreateEscrow
# TopUp

