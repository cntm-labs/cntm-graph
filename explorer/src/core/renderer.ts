/**
 * WebGPU Renderer Core
 * Handles device initialization and the main render loop for the Graph Explorer.
 */

export class GraphRenderer {
  private canvas: HTMLCanvasElement;
  private adapter: GPUAdapter | null = null;
  private device: GPUDevice | null = null;
  private context: GPUCanvasContext | null = null;
  private format: GPUTextureFormat = "bgra8unorm";

  constructor(canvas: HTMLCanvasElement) {
    this.canvas = canvas;
  }

  /**
   * Initializes the WebGPU device and context.
   */
  async init(): Promise<boolean> {
    if (!navigator.gpu) {
      console.error("WebGPU is not supported on this browser.");
      return false;
    }

    this.adapter = await navigator.gpu.requestAdapter();
    if (!this.adapter) {
      console.error("No appropriate GPUAdapter found.");
      return false;
    }

    this.device = await this.adapter.requestDevice();
    this.context = this.canvas.getContext("webgpu") as unknown as GPUCanvasContext;

    if (!this.context) {
      console.error("Failed to get WebGPU context.");
      return false;
    }

    this.format = navigator.gpu.getPreferredCanvasFormat();

    this.context.configure({
      device: this.device,
      format: this.format,
      alphaMode: "premultiplied",
    });

    console.log("WebGPU Initialized Successfully");
    return true;
  }

  /**
   * Starts the main render loop.
   */
  start() {
    if (!this.device || !this.context) return;

    const frame = () => {
      this.render();
      requestAnimationFrame(frame);
    };

    requestAnimationFrame(frame);
  }

  /**
   * Performs the actual rendering work for a single frame.
   */
  private render() {
    if (!this.device || !this.context) return;

    const commandEncoder = this.device.createCommandEncoder();
    const textureView = this.context.getCurrentTexture().createView();

    const renderPassDescriptor: GPURenderPassDescriptor = {
      colorAttachments: [
        {
          view: textureView,
          clearValue: { r: 0.02, g: 0.02, b: 0.02, a: 1.0 }, // #050505 approximate
          loadOp: "clear",
          storeOp: "store",
        },
      ],
    };

    const passEncoder = commandEncoder.beginRenderPass(renderPassDescriptor);
    // Future: Draw nodes and edges here
    passEncoder.end();

    this.device.queue.submit([commandEncoder.finish()]);
  }
}
