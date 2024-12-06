import { DragGesture } from "@use-gesture/vanilla";

// Connection to the WebSocket server
const socket = new WebSocket(`ws://${location.hostname}:9001`); // Replace with your actual server address

socket.binaryType = "arraybuffer"; // Ensure the socket is set to handle binary data if needed

socket.onclose = () => {
  setTimeout(() => {
    window.location.reload();
  }, 1000);
};

socket.onerror = (error) => console.error("WebSocket Error:", error);

// Function to send binary messages via WebSocket
function sendBinaryMessage(msgType, payloadBuffer) {
  // Create a new ArrayBuffer with 1 byte for msgType + payload
  const buffer = new ArrayBuffer(1 + payloadBuffer.byteLength);
  const view = new DataView(buffer);

  // Set the message type
  view.setUint8(0, msgType);

  // Set the payload
  new Uint8Array(buffer, 1).set(new Uint8Array(payloadBuffer));

  // Send the binary message
  socket.send(buffer);
}

// Function to create a buffer for Move messages
function createMoveBuffer(dx, dy) {
  const buffer = new ArrayBuffer(8); // 4 bytes for dx and 4 bytes for dy
  const view = new DataView(buffer);
  view.setInt32(0, dx, true); // true for little-endian
  view.setInt32(4, dy, true);
  return buffer;
}

// Function to create a buffer for Click messages
function createClickBuffer(button) {
  const buffer = new ArrayBuffer(1); // 1 byte for button
  const view = new DataView(buffer);
  view.setUint8(0, button);
  return buffer;
}

// Gesture area element
const gestureArea = document.body;

// Movement accumulator
let moveAccumulator = { x: 0, y: 0 };
let movePending = false;

// More efficient throttled send function using requestAnimationFrame
const throttledSendMove = (dx, dy) => {
  moveAccumulator.x += dx;
  moveAccumulator.y += dy;

  if (!movePending) {
    movePending = true;
    requestAnimationFrame(() => {
      if (moveAccumulator.x !== 0 || moveAccumulator.y !== 0) {
        const moveBuffer = createMoveBuffer(
          Math.round(moveAccumulator.x),
          Math.round(moveAccumulator.y)
        );
        sendBinaryMessage(0x01, moveBuffer);
        moveAccumulator.x = 0;
        moveAccumulator.y = 0;
      }
      movePending = false;
    });
  }
};

let lastDx = null;
let lastDy = null;

// Drag Gesture for mouse movement
new DragGesture(
  gestureArea,
  ({ delta: [dx, dy], active, movement: [mx, my] }) => {
    if (active) {
      // Calculate the movement since last update
      const deltaX = mx - (lastDx || 0);
      const deltaY = my - (lastDy || 0);

      if (deltaX !== 0 || deltaY !== 0) {
        throttledSendMove(deltaX, deltaY);
        lastDx = mx;
        lastDy = my;
      }
    } else {
      lastDx = null;
      lastDy = null;
    }
  }
);

// Click Event Handlers with proper button mapping
const handleMouseEvent = (e) => {
  e.preventDefault();
  // Map browser button values to our protocol values
  const buttonMap = {
    0: 1,  // Left click (browser) -> 1 (our protocol)
    2: 2   // Right click (browser) -> 2 (our protocol)
  };
  
  const button = buttonMap[e.button];
  if (button !== undefined) {
    const clickBuffer = createClickBuffer(button);
    sendBinaryMessage(0x03, clickBuffer);
  }
};

// Single event listeners for all mouse events
gestureArea.addEventListener("click", handleMouseEvent);
gestureArea.addEventListener("contextmenu", handleMouseEvent);

// Prevent default browser behavior
gestureArea.addEventListener("mousedown", (e) => e.preventDefault());
gestureArea.addEventListener("dragstart", (e) => e.preventDefault());
