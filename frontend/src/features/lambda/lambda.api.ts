import ky from "ky";

const LambdaApi = (baseURL: string) => {
  const client = ky.extend({
    prefixUrl: baseURL,
  });

  const getStates = () => client.get(`states`);
  const getParams = () => client.get(`params`);

  return { getStates, getParams };
};

export default LambdaApi;
