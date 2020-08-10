import { Chip8 } from "wasm-chip8";
import { memory } from "wasm-chip8/wasm_chip8_bg";

const WHITE = "#FFFFFF";
const BLACK = "#000000";
const PIXEL_SIZE = 10;
const width = 64;
const height = 32;


let chip8 = Chip8.new();
chip8.load_rom("bctest");

const canvas = document.getElementById("screen");
canvas.width = width * PIXEL_SIZE;
canvas.height = height * PIXEL_SIZE;

const ctx = canvas.getContext("2d");

function renderLoop()  {
    for (let i = 0; i < 10; i++) {
        chip8.tick();
    }
    chip8.decrement_timers();
    drawPixels();
    requestAnimationFrame(renderLoop);
}

function drawPixels() {
    const displayPtr = chip8.display_buffer_ptr();
    const display_size = chip8.display_buffer_size();
    const pixels = new Uint8Array(memory.buffer, displayPtr, display_size);
    // console.log(pixels);
    // debugger;

    ctx.beginPath();
    for (let row = 0; row < height; row++) {
        for (let col = 0; col < width; col++) {
            const idx = getIndex(row, col);

            ctx.fillStyle = pixelIsSet(idx, pixels) ? WHITE : BLACK;
            ctx.fillRect(
                col * PIXEL_SIZE,
                row * PIXEL_SIZE,
                PIXEL_SIZE,
                PIXEL_SIZE,
            )
        }
    }
    ctx.stroke();
}

function getIndex(row, col) {
    return row * width + col;
}

function pixelIsSet(idx, pixels) {
    let byte = Math.floor(idx/8);
    let mask = 0b10000000 >> idx % 8;
    return (pixels[byte] & mask) === mask;
}

renderLoop();
