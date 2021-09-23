import ky from "ky";
import { LamabdaParamEntry, ParamType } from "./lambda.types";

const LambdaApi = (baseURL: string) => {
  const client = ky.extend({
    prefixUrl: baseURL,
    timeout: 1000,
  });

  const getStates = () => client.get(`states`);
  const getParams = () => client.get(`params`);

  const setLambdaState = (state: string) => {
    const entries: LamabdaParamEntry[] = [
      {
        group: 'params',
        key: "state",
        type: ParamType.String,
        value: state,
      },
    ];
    return client.post(`params`, { json: entries, mode: "cors" });
  };

  return { getStates, getParams, setLambdaState };
};

export default LambdaApi;
