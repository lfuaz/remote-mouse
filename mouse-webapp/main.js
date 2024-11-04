import Hammer from "hammerjs";

document.addEventListener("DOMContentLoaded", function () {
  // Establish WebSocket connection with reconnection logic
  let socket;
  let isSocketOpen = false;

  const connectWebSocket = () => {
    socket = new WebSocket(`ws://${location.hostname}:8080`);

    socket.addEventListener("open", function () {
      console.log("WebSocket connection established.");
      isSocketOpen = true;
    });

    socket.addEventListener("error", function (event) {
      console.error("WebSocket error:", event);
      isSocketOpen = false;
    });

    socket.addEventListener("close", function () {
      console.warn("WebSocket connection closed. Reconnecting...");
      isSocketOpen = false;
      setTimeout(connectWebSocket, 1000); // Try to reconnect every second
    });
  };

  connectWebSocket();

  const sensitivityFactor = 1.2; // Adjusted for smoother cursor movement

  // Create touchpad overlay
  const touchpad = document.createElement("div");
  touchpad.id = "touchpad";
  touchpad.style.position = "fixed";
  touchpad.style.top = "0";
  touchpad.style.left = "0";
  touchpad.style.width = "100dvw";
  touchpad.style.height = "100dvh";
  touchpad.style.touchAction = "none"; // Disable default touch actions
  document.body.appendChild(touchpad);

  // Visual feedback element
  const feedbackCircle = document.createElement("div");
  feedbackCircle.id = "feedback-circle";
  feedbackCircle.style.position = "absolute";
  feedbackCircle.style.width = "50px";
  feedbackCircle.style.height = "50px";
  feedbackCircle.style.background = "rgba(0, 0, 0, 0.3)";
  feedbackCircle.style.borderRadius = "50%";
  feedbackCircle.style.transform = "translate(-50%, -50%)";
  feedbackCircle.style.pointerEvents = "none";
  feedbackCircle.style.display = "none";
  feedbackCircle.style.zIndex = "1000"; // Ensure it's on top
  document.body.appendChild(feedbackCircle);

  // Initialize Hammer.js
  const mc = new Hammer.Manager(touchpad);

  // Add recognizers
  const singlePan = new Hammer.Pan({ event: "singlepan", pointers: 1 });
  const doublePan = new Hammer.Pan({
    event: "doublepan",
    pointers: 2,
    threshold: 0,
  });
  const tap = new Hammer.Tap({ event: "singletap", taps: 1 });
  const press = new Hammer.Press({ event: "press", time: 600 });
  const doubleTap = new Hammer.Tap({
    event: "doubletap",
    taps: 2,
    interval: 300,
  });

  // Set up recognizer dependencies and drop recognizers
  doubleTap.recognizeWith(tap);
  tap.requireFailure(doubleTap);

  mc.add([singlePan, doublePan, tap, press, doubleTap]);

  let isMoving = false;
  let previousPosition = { x: 0, y: 0 };

  // Single finger pan for mouse movement
  mc.on("singlepanstart", function (event) {
    isMoving = true;
    previousPosition = { x: event.center.x, y: event.center.y };
    updateFeedbackCircle(event.center.x, event.center.y);
  });

  mc.on("singlepanmove", function (event) {
    if (!isMoving) return;

    const deltaX = (event.center.x - previousPosition.x) * sensitivityFactor;
    const deltaY = (event.center.y - previousPosition.y) * sensitivityFactor;
    previousPosition = { x: event.center.x, y: event.center.y };

    sendMovement(deltaX, deltaY);
    updateFeedbackCircle(event.center.x, event.center.y);
  });

  mc.on("singlepanend", function () {
    isMoving = false;
    feedbackCircle.style.display = "none";
  });

  // Two-finger pan for scrolling
  mc.on("doublepanstart", function (event) {
    previousPosition = { x: event.center.x, y: event.center.y };
  });

  mc.on("doublepanmove", function (event) {
    const deltaY = (event.center.y - previousPosition.y) * sensitivityFactor;
    previousPosition = { x: event.center.x, y: event.center.y };

    sendScroll(deltaY);
  });

  // Single tap for left click
  mc.on("singletap", function (event) {
    sendClick("left");
    vibrateDevice(50);
  });

  // Press for right click
  mc.on("press", function (event) {
    sendClick("right");
    vibrateDevice(100);
  });

  // Double tap for double click (optional)
  mc.on("doubletap", function (event) {
    sendClick("double");
    vibrateDevice(50);
  });

  function sendMovement(delta_x, delta_y) {
    if (isSocketOpen) {
      try {
        const movementData = { msg_type: "move", delta_x, delta_y };
        socket.send(JSON.stringify(movementData));
      } catch (error) {
        console.error("Failed to send movement data:", error);
      }
    }
  }

  function sendClick(button) {
    if (isSocketOpen) {
      try {
        const clickData = { msg_type: "click", button };
        socket.send(JSON.stringify(clickData));
      } catch (error) {
        console.error("Failed to send click data:", error);
      }
    }
  }

  function sendScroll(delta_y) {
    if (isSocketOpen) {
      try {
        const scrollData = { msg_type: "scroll", delta_y };
        socket.send(JSON.stringify(scrollData));
      } catch (error) {
        console.error("Failed to send scroll data:", error);
      }
    }
  }

  function vibrateDevice(duration) {
    if ("vibrate" in navigator) {
      navigator.vibrate(duration);
    }
  }

  function updateFeedbackCircle(x, y) {
    feedbackCircle.style.left = `${x}px`;
    feedbackCircle.style.top = `${y}px`;
    feedbackCircle.style.display = "block";
  }
});
