import * as React from 'react';
import { State as AppState } from '../../state';

import './Overview.css';

interface StatsProps {
  appState: Readonly<AppState>;
}

export class Overview extends React.Component<StatsProps> {
  public render() {
    const { appState } = this.props;
    const overview = appState.chainOverview;
    return (
      <div><pre> {overview} </pre></div>
    )
  }
}
