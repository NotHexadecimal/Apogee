const canvas = document.querySelector('canvas');
const ctx = canvas.getContext('2d');
const pixelRatio = window.devicePixelRatio;

const state = {
  canvas: {
    width: 0,
    height: 0
  }
};

const resize = () => {
  canvas.width = window.innerWidth * pixelRatio;
  canvas.height = window.innerHeight * pixelRatio;
  canvas.style.width = `${window.innerWidth}px`;
  canvas.style.height = `${window.innerHeight}px`;
  ctx.scale(pixelRatio, pixelRatio);
  state.canvas.width = window.innerWidth;
  state.canvas.height = window.innerHeight;
};

const draw = () => {
  ctx.beginPath();
  ctx.moveTo(0, 0);
  ctx.lineTo(state.canvas.width, state.canvas.height);
  ctx.stroke();
};

const resizeDraw = () => {
  resize();
  draw();
};

addEventListener('resize', resizeDraw);

resizeDraw();
