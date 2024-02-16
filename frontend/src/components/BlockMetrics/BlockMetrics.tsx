// Source code for the Substrate Telemetry Server.
// Copyright (C) 2022 Parity Technologies (UK) Ltd.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

import * as React from 'react';
import { State as AppState } from '../../state';
import {
  Chart as ChartJS,
  CategoryScale,
  LinearScale,
  PointElement,
  LineElement,
  Title,
  Tooltip,
  Legend,
} from 'chart.js';

import { Line } from 'react-chartjs-2';
import { NodeId, SingleBlockMetricDetails } from 'src/common/types';
import './BlockMetrics.css';

interface StatsProps {
  appState: Readonly<AppState>;
}

export class BlockMetrics extends React.Component<StatsProps> {
  public render() {
    const { appState } = this.props;
    const stats = appState.blockMetricsStats;

    ChartJS.register(
      CategoryScale,
      LinearScale,
      PointElement,
      LineElement,
      Title,
      Tooltip,
      Legend
    );


    let arrays = [...stats.metrics];
    let lines = arrays.map(v => 
      [display_block_metric_graph([...v[1]], v[0]), display_block_network_delay_graph([...v[1]], v[0], appState.blockMetricsStats.bestBlockTimes)]
    );

    let a = lines.map(v => <div className='rowC'> {v} </div>);

    return <div style={{height: 300, width: 500}}>
      {a}
    </div>
    
  }
}

function display_block_network_delay_graph(metrics: [number,  SingleBlockMetricDetails][], id: NodeId, bestTimes: Map<number, number>): JSX.Element {
  metrics.sort((a, b) => a[0] - b[0])

  const options = {
    responsive: true,
    maintainAspectRatio: false,
    scales: {
      y: {
        beginAtZero: true,
        border: {  
          color: '#777777'
        },
        grid: {  
          color: '#777777'
        },
        ticks: {
          color: '#FFFFFF'
        }           
      },
      x: {
        border: {  
          color: '#777777'
        },
        grid: {  
          color: '#777777'
        },
        ticks: {
          color: '#FFFFFF'
        }             
      },
    },
    plugins: {
      legend: {
        position: 'top' as const,
      },
      title: {
        display: true,
        text: 'Block Metrics for id: ' + id,
      },
    },
  };

  const labels = metrics.map((v) => v[0]);
  const syncTimeDataset: (number | undefined)[] = [];
  const importTimeDataset: (number | undefined)[] = [];
  const totalTimeDataset: (number | undefined)[] = [];
  metrics.forEach(v => {
    let block_best_time = bestTimes.get(v[0]) ?? 0;

    if (block_best_time == 0) {
      console.log("Not Best time Was found! FIXME");
      syncTimeDataset.push(0);
      importTimeDataset.push(0);
      totalTimeDataset.push(0);
    }
    else if (v[1].proposal != undefined) {
      syncTimeDataset.push(0);
      importTimeDataset.push(0);
      totalTimeDataset.push(0);
    }
    else if (v[1].import_block && v[1].sync_block ) {
      syncTimeDataset.push(v[1].sync_block.start_timestamp - block_best_time);
      importTimeDataset.push(v[1].import_block.start_timestamp - block_best_time);
      totalTimeDataset.push(v[1].import_block.end_timestamp - block_best_time);
    } else {
      console.log("This should not be visible!!!! FIXME");
      syncTimeDataset.push(0);
      importTimeDataset.push(0);
      totalTimeDataset.push(0);
    }
  })

  const data = {
    labels,
    datasets: [
      {
        label: 'Total Propagation',
        data: totalTimeDataset,
        borderColor: 'rgb(125, 255, 125)',
        backgroundColor: 'rgba(125, 255, 125, 0.5)',
      },
      {
        label: 'Sync Delay',
        data: syncTimeDataset,
        borderColor: 'rgb(0, 255, 255)',
        backgroundColor: 'rgba(0, 255, 255, 0.5)',
      },
      {
        label: 'Import Delay',
        data: importTimeDataset,
        borderColor: 'rgb(255, 0, 255)',
        backgroundColor: 'rgba(255, 0, 255, 0.5)',
      }
    ]
  };

  return <div style={{height: 300, width: 500}}>
    <Line options = {options} data = {data}/>
  </div>

}

function display_block_metric_graph(metrics: [number, SingleBlockMetricDetails][], id: NodeId): JSX.Element {
  metrics.sort((a, b) => a[0] - b[0])

  const options = {
    responsive: true,
    maintainAspectRatio: false,
    scales: {
      y: {
        beginAtZero: true,
        border: {  
          color: '#777777'
        },
        grid: {  
          color: '#777777'
        },
        ticks: {
          color: '#FFFFFF'
        }           
      },
      x: {
        border: {  
          color: '#777777'
        },
        grid: {  
          color: '#777777'
        },
        ticks: {
          color: '#FFFFFF'
        }             
      },
    },
    plugins: {
      legend: {
        position: 'top' as const,
      },
      title: {
        display: true,
        text: 'Block Metrics for id: ' + id,
      },
    },
  };

  const labels = metrics.map((v) => v[0]);
  const syncTimeDataset: (number | undefined)[] = [];
  const importTimeDataset: (number | undefined)[] = [];
  const proposalTimeDataset: (number | undefined)[] = [];
  const totalTimeDataset: (number | undefined)[] = [];
  metrics.forEach(v => {
    let totalTime = 0;
    totalTime += v[1].sync_block?.duration ?? 0;
    syncTimeDataset.push(v[1].sync_block?.duration)

    totalTime += v[1].import_block?.duration ?? 0;
    importTimeDataset.push(v[1].import_block?.duration)

    totalTime += v[1].proposal?.duration ?? 0;
    proposalTimeDataset.push(v[1].proposal?.duration)

    totalTimeDataset.push(totalTime)
  })

  const data = {
    labels,
    datasets: [
      {
        label: 'Total Time',
        data: totalTimeDataset,
        borderColor: 'rgb(125, 255, 125)',
        backgroundColor: 'rgba(125, 255, 125, 0.5)',
      },
      {
        label: 'Sync Time',
        data: syncTimeDataset,
        borderColor: 'rgb(0, 255, 255)',
        backgroundColor: 'rgba(0, 255, 255, 0.5)',
      },
      {
        label: 'Import Time',
        data: importTimeDataset,
        borderColor: 'rgb(255, 0, 255)',
        backgroundColor: 'rgba(255, 0, 255, 0.5)',
      },
      {
        label: 'Proposal Time',
        data: proposalTimeDataset,
        borderColor: 'rgb(255, 255, 0)',
        backgroundColor: 'rgba(255, 255, 0, 0.5)',
      }
    ]
  };

  return <div style={{height: 300, width: 500}}>
    <Line options = {options} data = {data}/>
  </div>
}