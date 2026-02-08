import init, { transform2_known_crs } from "../npm/proj_lite_web.js";

const resultEl = document.getElementById("result");
const runBtn = document.getElementById("run");

await init();

runBtn.addEventListener("click", () => {
  const from = document.getElementById("from").value;
  const to = document.getElementById("to").value;
  const x = Number(document.getElementById("x").value);
  const y = Number(document.getElementById("y").value);

  try {
    const out = transform2_known_crs(from, to, x, y);
    resultEl.textContent = JSON.stringify({ input: [x, y], output: out }, null, 2);
  } catch (err) {
    resultEl.textContent = `Error: ${err}`;
  }
});
