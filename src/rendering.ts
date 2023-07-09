export type TrainPosition = {
  x: number;
  y: number;
};

export const renderCanvas = (
  canvas: HTMLCanvasElement,
  ...trains: TrainPosition[]
) => {
  const ctx = canvas.getContext("2d");

  // Set canvas width and height
  canvas.width = 1000;
  canvas.height = 600;

  // Calculate the coordinates and dimensions for the rectangles
  const rectWidth = 400;
  const rectHeight = 200;
  const rectMargin = 100;

  if (ctx) {
    // Draw the rectangles in the four quadrants
    // Top-left quadrant
    ctx.strokeStyle = "red";
    ctx.lineWidth = 5;
    ctx.strokeRect(rectMargin, rectMargin, rectWidth, rectHeight);

    // Top-right quadrant
    ctx.strokeStyle = "blue";
    ctx.strokeRect(
      canvas.width - rectWidth - rectMargin,
      rectMargin,
      rectWidth,
      rectHeight
    );

    // Bottom-left quadrant
    ctx.strokeStyle = "green";
    ctx.strokeRect(
      rectMargin,
      canvas.height - rectHeight - rectMargin,
      rectWidth,
      rectHeight
    );

    // Bottom-right quadrant
    ctx.strokeStyle = "yellow";
    ctx.strokeRect(
      canvas.width - rectWidth - rectMargin,
      canvas.height - rectHeight - rectMargin,
      rectWidth,
      rectHeight
    );

    const trainColors = ["pink", "brown", "orange", "purple"];

    trains.forEach((train, index) =>
      renderTrain(ctx, train, trainColors[index])
    );
  }
};

function renderTrain(
  ctx: CanvasRenderingContext2D,
  train: TrainPosition,
  color: string
) {
  console.log("rendering train", train);
  ctx.fillStyle = color;
  ctx.fillRect(train.x - 5, train.y - 5, 10, 10);
}
