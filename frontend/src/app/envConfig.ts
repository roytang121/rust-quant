
export enum Env {
  Local = 'Local',
  Tokyo = 'Tokyo',
}

export interface EnvConfig {
  name: Env,
  grpc_host: string,
  instances: string[],
}

export const staticEnvConfig: EnvConfig[]= [
  {
    name: Env.Local,
    grpc_host: 'http://localhost:50051',
    instances: ['swap-mm-ethusd', 'latency-mm'],
  },
  {
    name: Env.Tokyo,
    grpc_host: 'https://quant.roytang.me/grpc',
    instances: ['swap-mm-ethusd', 'latency-mm'],
  }
]
