import { Button, Navbar, NavbarDivider, NavbarGroup } from "@blueprintjs/core";
import { ColDef } from "ag-grid-community";
import "ag-grid-enterprise";
import { AgGridReact } from "ag-grid-react";
import { memo, useEffect, useMemo, useState } from "react";
import { combineLatest, switchMap, switchMapTo, timer } from "rxjs";
import LambdaApi from "./lambda.api";
import { LamabdaParamEntry } from "./lambda.types";

import { SubscribeRequest } from 'redis-grpc/gen-js/redis_grpc_pb';
import { RedisGrpcPromiseClient } from 'redis-grpc/gen-js/redis_grpc_grpc_web_pb';

interface Props {
  host: string;
}

const Lambda = ({ host }: Props) => {
  const [api] = useState(LambdaApi(host));
  const [entries, setEntries] = useState<LamabdaParamEntry[]>([]);

  useEffect(() => {
    const service = new RedisGrpcPromiseClient("http://localhost:50051")

    let sub_request = new SubscribeRequest();
    sub_request.setChannels("StrategyStates:swap-mm-ethusd");
    let pubsub = service.subscribe(sub_request);
    pubsub.on("data", data => {
      const message = data.getMessage();
      console.log(message);
      
      const states: LamabdaParamEntry[] = JSON.parse(message);
      setEntries([...states])
    })

    return () => {
      pubsub.cancel()
    }
  }, [])
  
  useEffect(() => {
    const sub = timer(0, 1000)
      .pipe(switchMap(t => {
        return combineLatest([api.getStates(), api.getParams()])
      }))
      .subscribe(async ([state_response, params_response]) => {
        const states: LamabdaParamEntry[] = await state_response.json();
        const params: LamabdaParamEntry[] = await params_response.json();
        setEntries([...states, ...params]);
      });
    return () => sub.unsubscribe();
  }, [api]);

  const colDefs = useMemo((): ColDef[] => {
    return [
      { field: "group", enableRowGroup: true, rowGroup: true, hide: true },
      { field: "key" },
      { field: "value", editable: true },
      { field: "type" },
    ];
  }, []);

  const getRowId = (entry: LamabdaParamEntry) => entry.key;

  return (
    <>
      <Navbar>
        <NavbarGroup>
          <Button
            text="Start"
            intent="primary"
            onClick={() => api.setLambdaState("Live")}
          />
          <NavbarDivider />
          <Button
            text="Stop"
            intent="danger"
            onClick={() => api.setLambdaState("Stopped")}
          />
        </NavbarGroup>
      </Navbar>
      <AgGridReact
        immutableData
        columnDefs={colDefs}
        rowData={entries}
        getRowNodeId={getRowId}
        groupDefaultExpanded={1}
      />
    </>
  );
};

export default memo(Lambda);
