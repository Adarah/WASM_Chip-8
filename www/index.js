import { Chip8 } from "wasm-chip8";
import { memory } from "wasm-chip8/wasm_chip8_bg";

const WHITE = "#FFFFFF";
const BLACK = "#000000";
const PIXEL_SIZE = 10;
const width = 64;
const height = 32;

let chip8 = Chip8.new();
chip8.load_rom("tetris");

const canvas = document.getElementById("screen");
canvas.width = width * (PIXEL_SIZE + 10);
canvas.height = height * (PIXEL_SIZE + 10);

const ctx = canvas.getContext("2d");

function renderLoop() {
  for (let i = 0; i < 10; i++) {
    //     debugger;
    chip8.tick();
    // debugger;
  }
  chip8.decrement_timers();
  drawPixels();
  fps.render();
  requestAnimationFrame(renderLoop);
}

function drawPixels() {
  const displayPtr = chip8.display_buffer_ptr();
  const display_size = chip8.display_buffer_size();
  const pixels = new Uint8Array(memory.buffer, displayPtr, display_size);

  ctx.beginPath();
  for (let row = 0; row < height; row++) {
    for (let col = 0; col < width; col++) {
      const idx = getIndex(row, col);

      ctx.fillStyle = pixelIsSet(idx, pixels) ? WHITE : BLACK;
      ctx.fillRect(col * PIXEL_SIZE, row * PIXEL_SIZE, PIXEL_SIZE, PIXEL_SIZE);
    }
  }
  ctx.stroke();
}

function getIndex(row, col) {
  return row * width + col;
}

function pixelIsSet(idx, pixels) {
  let byte = Math.floor(idx / 8);
  let mask = 0b10000000 >> idx % 8;
  return (pixels[byte] & mask) === mask;
}

function translate_key(keycode) {
  let map = {
    Digit1: 1,
    Digit2: 2,
    Digit3: 3,
    KeyQ: 4,
    KeyW: 5,
    KeyE: 6,
    KeyA: 7,
    KeyS: 8,
    KeyD: 9,
    KeyZ: 0xa,
    KeyX: 0,
    KeyC: 0xb,
    Digit4: 0xc,
    KeyR: 0xd,
    KeyF: 0xe,
    KeyV: 0xf,
  };
  return map[keycode];
}

window.addEventListener("keydown", function (event) {
    let key = translate_key(event.code);
    if (typeof key !== 'undefined') {
        chip8.press_key(translate_key(event.code));
    }
});

window.addEventListener("keyup", function (event) {
    let key = translate_key(event.code);
    if (typeof key !== 'undefined') {
        chip8.release_key(translate_key(event.code));
    }
});

const fps = new class {
  constructor() {
    this.fps = document.getElementById("fps");
    this.frames = [];
    this.lastFrameTimeStamp = performance.now();
  }

  render() {
    // Convert the delta time since the last frame render into a measure
    // of frames per second.
    const now = performance.now();
    const delta = now - this.lastFrameTimeStamp;
    this.lastFrameTimeStamp = now;
    const fps = 1 / delta * 1000;

    // Save only the latest 100 timings.
    this.frames.push(fps);
    if (this.frames.length > 100) {
      this.frames.shift();
    }

    // Find the max, min, and mean of our 100 latest timings.
    let min = Infinity;
    let max = -Infinity;
    let sum = 0;
    for (let i = 0; i < this.frames.length; i++) {
      sum += this.frames[i];
      min = Math.min(this.frames[i], min);
      max = Math.max(this.frames[i], max);
    }
    let mean = sum / this.frames.length;

    // Render the statistics.
    this.fps.textContent = `
Frames per Second:
         latest = ${Math.round(fps)}
avg of last 100 = ${Math.round(mean)}
min of last 100 = ${Math.round(min)}
max of last 100 = ${Math.round(max)}
`.trim();
  }
};

renderLoop();
