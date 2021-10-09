import { Button, Navbar, NavbarDivider, NavbarGroup } from "@blueprintjs/core";
import { CellEditingStoppedEvent, ColDef } from "ag-grid-community";
import "ag-grid-enterprise";
import { AgGridReact } from "ag-grid-react";
import { memo, useCallback, useEffect, useMemo, useState } from "react";
import { RedisGrpcPromiseClient } from "redis-grpc/gen-js/redis_grpc_grpc_web_pb";
import {
  PublishRequest,
  SubscribeRequest,
} from "redis-grpc/gen-js/redis_grpc_pb";
import { LamabdaParamEntry, ParamType } from "./lambda.types";
import { entryValuePaser } from "./valueParser";

interface Props {
  host: string;
  instance: string;
}

const Lambda = ({ host, instance }: Props) => {
  const [stateEntries, setStateEntries] = useState<LamabdaParamEntry[]>([]);
  const [paramEntries, setParamEntries] = useState<LamabdaParamEntry[]>([]);

  useEffect(() => {
    setStateEntries([])
    setParamEntries([])
  }, [host, instance])

  useEffect(() => {
    const service = new RedisGrpcPromiseClient(host);

    let sub_request = new SubscribeRequest();
    sub_request.setChannelsList([`StrategyStates:${instance}`]);
    let pubsub = service.subscribe(sub_request);
    pubsub.on("data", (data) => {
      const message = data.getMessage();
      const states: LamabdaParamEntry[] = JSON.parse(message);
      setStateEntries([...states]);
    });

    return () => {
      pubsub.cancel();
    };
  }, [host, instance]);

  useEffect(() => {
    const service = new RedisGrpcPromiseClient(host);

    let sub_request = new SubscribeRequest();
    sub_request.setChannelsList([`StrategyParams:${instance}`]);
    let pubsub = service.subscribe(sub_request);
    pubsub.on("data", (data) => {
      const message = data.getMessage();
      const params: LamabdaParamEntry[] = JSON.parse(message);
      setParamEntries([...params]);
    });

    return () => {
      pubsub.cancel();
    };
  }, [host, instance]);

  const updateParam = useCallback((entry: LamabdaParamEntry) => {
    const service = new RedisGrpcPromiseClient(host);
    const pub_request = new PublishRequest();
    pub_request.setChannel(`UpdateParam:${instance}`);
    pub_request.setMessage(JSON.stringify(entry));
    console.log(pub_request);
    service.publish(pub_request).then(console.log).catch(console.error);
  }, []);

  const colDefs = useMemo((): ColDef[] => {
    return [
      { field: "group", enableRowGroup: true, rowGroup: true, hide: true },
      { field: "key" },
      {
        field: "value",
        editable: (params) => params.data.group === "params",
        valueParser: entryValuePaser,
      },
      { field: "type" },
    ];
  }, []);

  const getRowId = (entry: LamabdaParamEntry) => entry.key;

  const onCellEditingStopped = useCallback(
    (params: CellEditingStoppedEvent) => {
      console.log(params.data);
      updateParam(params.data);
    },
    [updateParam]
  );

  return (
    <>
      <Navbar>
        <NavbarGroup>
          <Button
            icon="play"
            text="Start"
            intent="primary"
            onClick={() =>
              updateParam({
                key: "state",
                group: "params",
                value: "Live",
                type: ParamType.String,
              })
            }
          />
          <NavbarDivider />
          <Button
            icon="stop"
            text="Stop"
            intent="danger"
            onClick={() =>
              updateParam({
                key: "state",
                group: "params",
                value: "Stopped",
                type: ParamType.String,
              })
            }
          />
        </NavbarGroup>
      </Navbar>
      <AgGridReact
        immutableData
        columnDefs={colDefs}
        rowData={[...stateEntries, ...paramEntries]}
        getRowNodeId={getRowId}
        groupDefaultExpanded={1}
        onCellEditingStopped={onCellEditingStopped}
      />
    </>
  );
};

export default memo(Lambda);
