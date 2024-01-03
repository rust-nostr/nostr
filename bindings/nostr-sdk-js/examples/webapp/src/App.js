import React, { Component } from "react";
import { Client, ClientSigner, Nip07Signer, initLogger, loadWasmAsync } from '@rust-nostr/nostr-sdk'
import './App.css';

class App extends Component {
  constructor() {
    super();
    this.state = { 
      public_key: null,
      nip07_signer: null,
      client: null
    };
  }

  async componentDidMount() {
    // Load was in async mode
    await loadWasmAsync();

    // Try to initialize log
    try {
      initLogger();
    } catch (error) {}
  }

  handleLogin = async () => {
    try {
      // Get NIP07 signer and compose Client with ClientSigner
      let nip07_signer = new Nip07Signer();
      let signer = ClientSigner.nip07(nip07_signer);
      let client = new Client(signer);

      let public_key = await nip07_signer.getPublicKey();
  
      // Add relays
      await client.addRelay("wss://relay.damus.io");
      await client.addRelay("wss://nos.lol");
      await client.addRelay("wss://nostr.oxtr.dev");
  
      // Connect to relays
      await client.connect();
  
      // Save client to state
      this.setState({ client, nip07_signer, public_key });
    } catch (error) {
        console.log(error) 
    }
  };

  handlePublishTextNote = async () => {
    try {
      await this.state.client.publishTextNote("Test from Rust Nostr SDK JavaScript bindings with NIP07 signer!", []);
    } catch (error) {
        console.log(error) 
    }
  };

  handleLogout = async () => {
    try {
      await this.state.client.shutdown();
      this.setState({ client: null });
      console.log("Logout done");
      } catch (error) {
          console.log(error) 
      }
  };

  render() {
    if (this.state.client == null) {
      // Login page
      return (
        <div className="App">
          <header className="App-header">
            <button className="btn btn-primary" onClick={this.handleLogin}>
              Login
            </button>
          </header>
        </div>
      );
    } else {
      // Home page
      return (
        <div className="App">
          <header className="App-header">
            <p>Public key: {this.state.public_key.toBech32()}</p>
            <button className="btn btn-primary" onClick={this.handlePublishTextNote}>
              Publish text note
            </button>
            <button className="btn btn-primary" onClick={this.handleLogout}>
              Logout
            </button>
          </header>
        </div>
      );
    }
  }
}

export default App;
