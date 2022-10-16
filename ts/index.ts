const canvas = document.querySelector('canvas');
const ctx = canvas.getContext('2d');
const pixelRatio = window.devicePixelRatio;

const state = {
  canvas: {
    width: 0,
    height: 0
  }
};

const update = (dt: number): void => {

};

const draw = (): void => {
  ctx.clearRect(0, 0, state.canvas.width, state.canvas.height);

  ctx.beginPath();
  ctx.moveTo(0, 0);
  ctx.lineTo(state.canvas.width, state.canvas.height);
  ctx.stroke();
};

// event handlers
const resize = (): void => {
  canvas.width = window.innerWidth * pixelRatio;
  canvas.height = window.innerHeight * pixelRatio;
  canvas.style.width = `${window.innerWidth}px`;
  canvas.style.height = `${window.innerHeight}px`;
  ctx.scale(pixelRatio, pixelRatio);
  state.canvas.width = window.innerWidth;
  state.canvas.height = window.innerHeight;
};

addEventListener('resize', resize);
resize();

// game loop
let prevTime = 0;
requestAnimationFrame((time: number) => {
  prevTime = time;
  requestAnimationFrame(loop);
});
const loop = (time: number) => {
  const dt = time - prevTime;
  prevTime = time;

  update(dt);
  draw();

  requestAnimationFrame(loop);
};
