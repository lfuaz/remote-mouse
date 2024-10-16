# 🖱️ Remote Cursor - webPad

**Remote Cursor - webPad** allows you to control your PC cursor remotely through a smartphone web interface. Powered by a Rust web server, WebSocket technology, and HTTP, this app brings your computer cursor to your phone’s browser with real-time responsiveness. It's easy to set up, and you can get started with just a single executable file! 🖥️📲
## 🌐 Overview
With **webPad**, you can
- 🖱️ Move your cursor remotely
- 📱 Use any device with a browser as a virtual touchpad
- 🛠️ Enjoy the performance and speed of a Rust-powered server
## 🚀 Feature
- **One-click Start**: Just launch the `.exe` file on Windows or run `./your-executable` on Linux to get started.
- **Real-time Control**: The app uses WebSocket technology to provide instantaneous cursor feedback.- **Cross-device**: Works on any smartphone or tablet with a browser.
- **Efficient HTTP Hosting**: The web app is served locally, providing a secure and private connection.

---

## 📥 Installation

To install **Remote Cursor - webPad**:
1. Clone or download this repository.
2. Follow the build instructions below for either Windows or Linux.
---

## 🛠️ Building the App

### Prerequisites
- **npm**: Required to build the `mouse-webapp` frontend project.
- **Rust and Cargo**: Required to build the server backend.

### Steps

1. **Build the Web Interface**:
   - Open a terminal in the `mouse-webapp` project folder.
   - Run:
     ```bash
     npm install
     npm run build
     ```
   - Move the generated `build` folder into the `server` folder so that it can be served by the Rust application.

2. **Build the Rust Server**:
   - Navigate to the `server` folder where the Rust code resides.
   - Run:
     ```bash
     cargo build --release
     ```
   - This will produce an executable file in `target/release`.

### Running the Executable

- **On Windows**: Double-click the `.exe` file created in `target/release`.
- **On Linux**: Run the executable from the terminal:
  ```bash
  ./target/release/your-executable-name
  ```

Once the server starts, it will display a local HTTP address (e.g., `http://localhost:8000`). Open this address on your smartphone’s browser to begin controlling the PC cursor remotely!

---

## 🔧 Technical Details

- **Rust Web Server**: Manages both HTTP requests for the web app and WebSocket connections for real-time control.
- **WebSockets**: Powers instant communication between the web interface and the cursor.
- **HTTP Serving**: Hosts the `mouse-webapp` interface, accessible over a local network.

---

## 🎉 Enjoy the Freedom of Remote Control

**webPad** provides the flexibility to control your PC from anywhere in the room or even outside! Perfect for presentations, movie nights, or just a bit of convenient browsing.

--- 

## 🔗 Contributing

Feel free to open issues or submit pull requests to enhance **Remote Cursor - webPad**. Contributions are always welcome! 😄
