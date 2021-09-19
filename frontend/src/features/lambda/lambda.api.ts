import ky from "ky";

const LambdaApi = (baseURL: string) => {
  const client = ky.extend({
    prefixUrl: baseURL,
  });

  const getStates = () => client.get(`states`);

  return { getStates };
};

export default LambdaApi;
