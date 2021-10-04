import { ValueParserParams } from "ag-grid-community";
import { LamabdaParamEntry } from "./lambda.types";
import { toNumber, includes } from 'lodash-es';

export const entryValuePaser = (params: ValueParserParams): any => {
  let data: LamabdaParamEntry = params.data;
  switch (data.type) {
    case "String":
      return `${params.newValue}`;
    case "Int":
    case "Float":
      return toNumber(params.newValue);
    case "Bool":
      return includes([true, "true", 1, "1"], params.newValue)
    default:
      break;
  }
}