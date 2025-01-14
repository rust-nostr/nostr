import React, { Component } from "react";
import { ClientBuilder, EventBuilder, NostrSigner, NostrZapper, Filter, LogLevel, NegentropyOptions, BrowserSigner, NostrDatabase, PublicKey, ZapDetails, ZapEntity, ZapType, initLogger, loadWasmAsync } from '@rust-nostr/nostr-sdk'
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
      initLogger(LogLevel.info());
    } catch (error) {}
  }

  handleLogin = async () => {
    try {
      // Get NIP07 signer and compose Client with NostrSigner
      let nip07_signer = new BrowserSigner();
      let signer = NostrSigner.nip07(nip07_signer);
      let zapper = await NostrZapper.webln();
      let db = await NostrDatabase.indexeddb("nostr-sdk-webapp-example");
      let client = new ClientBuilder().signer(signer).zapper(zapper).database(db).build();

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

  handleReconcile = async () => {
    try {
      let filter = new Filter().author(this.state.public_key);
      let opts = new NegentropyOptions();
      await this.state.client.sync(filter, opts);
    } catch (error) {
        console.log(error)
    }
  }

  handleQueryDatabase = async () => {
    try {
      let filter = new Filter().author(this.state.public_key);
      let database = this.state.client.database;
      console.time("query");
      let events = await database.query([filter]);
      console.timeEnd("query");
      console.log("Got", events.length, "events");
    } catch (error) {
      console.log(error)
    }
  }

  handlePublishTextNote = async () => {
    try {
      let builder = EventBuilder.textNote("Test from rust-nostr JavaScript bindings with NIP07 signer!", []);
      await this.state.client.sendEventBuilder(builder);
    } catch (error) {
        console.log(error)
    }
  };

  handleZap = async () => {
    try {
      let pk = PublicKey.fromBech32("npub1drvpzev3syqt0kjrls50050uzf25gehpz9vgdw08hvex7e0vgfeq0eseet");
      let entity = ZapEntity.publicKey(pk);
      let details = new ZapDetails(ZapType.Public).message("Zap for Rust Nostr!");
      await this.state.client.zap(entity, 1000, details);
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
            <button className="btn btn-primary" onClick={this.handleReconcile}>
              Negentropy reconciliation
            </button>
            <button className="btn btn-primary" onClick={this.handleQueryDatabase}>
              Query local database
            </button>
            <button className="btn btn-primary" onClick={this.handlePublishTextNote}>
              Publish text note
            </button>
            <button className="btn btn-primary" onClick={this.handleZap}>
              Zap!
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
