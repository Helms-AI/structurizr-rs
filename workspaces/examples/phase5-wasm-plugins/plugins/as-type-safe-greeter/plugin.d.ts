declare namespace __AdaptedExports {
  /** Exported memory */
  export const memory: WebAssembly.Memory;
  /**
   * assembly/index/main
   */
  export function main(): void;
  /**
   * assembly/index/run
   */
  export function run(): void;
  /**
   * assembly/index/getHeapSize
   * @returns `i32`
   */
  export function getHeapSize(): number;
  /**
   * assembly/index/getAllocatedMemory
   * @returns `i32`
   */
  export function getAllocatedMemory(): number;
}
/** Instantiates the compiled WebAssembly module with the given imports. */
export declare function instantiate(module: WebAssembly.Module, imports: {
  env: unknown,
}): Promise<typeof __AdaptedExports>;
