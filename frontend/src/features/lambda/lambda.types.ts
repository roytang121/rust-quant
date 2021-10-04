export enum ParamType {
  String = 'String',
  Int = 'Int',
  Float = 'Float',
  Bool = 'Bool',
}
export type LamabdaParamEntry = {
  group: 'states' | 'params',
  key: string,
  type: ParamType
  value: any,
}