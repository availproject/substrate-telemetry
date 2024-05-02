// Source code for the Substrate Telemetry Server.
// Copyright (C) 2023 Parity Technologies (UK) Ltd.
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
import { Maybe } from '../../../common';
import { ColumnProps } from './';
import { Node } from '../../../state';
import { milliOrSecond } from '../../../utils';
import icon from '../../../icons/dashboard.svg';

export class BlockSyncTimeColumn extends React.Component<ColumnProps> {
  public static readonly label = 'Block Sync Time';
  public static readonly icon = icon;
  public static readonly width = 58;
  public static readonly setting = 'blocksync';
  public static readonly sortBy = ({ syncTime }: Node) =>
    syncTime == null ? Infinity : syncTime;

  private data: Maybe<number>;

  public shouldComponentUpdate(nextProps: ColumnProps) {
    return this.data !== nextProps.node.syncTime;
  }

  render() {
    const { syncTime } = this.props.node;
    const print =
    syncTime == null ? '∞' : milliOrSecond(syncTime);

    this.data = syncTime;

    return <td className="Column">{print}</td>;
  }
}
