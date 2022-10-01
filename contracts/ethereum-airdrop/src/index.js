import _ from 'web3';

var Web3 = require('web3');
// var Accounts = require('web3-eth-accounts');

// var accounts = new Accounts('ws://localhost:8546');

const getEthereum = async function() {
    if (window.ethereum) {
        const web3 = new Web3(window.ethereum);
        const [eth_address] = await window.ethereum.enable();
        
        var message = "Testing string";
        // var hash = web3.utils.sha3(message);
        var accounts = await web3.eth.getAccounts();
        var signature = await web3.eth.personal.sign(message, accounts[0]);
        var signing_address = web3.eth.accounts.recover(message, signature);

        window.alert("Eth is now ready, account is " + accounts[0]);
        var sig_log = "Plaintext message: " + message + "\n\nSignature: " + signature + 
        "\n\nAddress that signed: " + signing_address;
        window.alert(sig_log);

    }
}
getEthereum();

