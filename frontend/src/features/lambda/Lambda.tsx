import { ColDef } from "ag-grid-community";
import { AgGridReact } from "ag-grid-react";
import { memo, useEffect, useMemo, useState } from "react";
import { timer } from "rxjs";
import LambdaApi from "./lambda.api";
import { LamabdaParamEntry } from "./lambda.types";

import "ag-grid-enterprise";

interface Props {
  host: string;
}

const Lambda = ({ host }: Props) => {
  const [api] = useState(LambdaApi(host));
  const [entries, setEntries] = useState<LamabdaParamEntry[]>([]);
  useEffect(() => {
    const sub = timer(0, 1000).subscribe(async (_) => {
      const state_response = await api.getStates();
      const params_response = await api.getParams();
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
    <AgGridReact
      immutableData
      columnDefs={colDefs}
      rowData={entries}
      getRowNodeId={getRowId}
      groupDefaultExpanded={1}
    />
  );
};

export default memo(Lambda);
