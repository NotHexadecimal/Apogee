const canvas = document.querySelector('canvas');
const ctx = canvas.getContext('2d');

const resizeDraw = () => {
  canvas.width = document.documentElement.clientWidth;
  canvas.height = document.documentElement.clientHeight;

  ctx.beginPath();
  ctx.moveTo(0, 0);
  ctx.lineTo(canvas.width, canvas.height);
  ctx.stroke();
}

addEventListener('resize', resizeDraw);

resizeDraw();
