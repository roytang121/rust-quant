export enum ParamType {
  String = 'String',
  Number = 'Number',
  Bool = 'Bool',
}
export type LamabdaParamEntry = {
  group: 'states' | 'params',
  key: string,
  type: ParamType
  value: any,
}