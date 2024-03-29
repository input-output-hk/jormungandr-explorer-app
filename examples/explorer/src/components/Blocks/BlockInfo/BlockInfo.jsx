import React from 'react';
import Table from 'react-bootstrap/Table';

import graphql from 'babel-plugin-relay/macro';
import { createFragmentContainer } from 'react-relay';

import './blockInfo.scss';
import BlockLink from '../../Commons/BlockLink/BlockLink';
import EpochLink from '../../Commons/EpochLink/EpochLink';

const BlockInfo = ({ block }) => (
  <div className="entityInfoTable">
    <h2>Block</h2>
    <div className="keyValueTable">
      <Table striped bordered hover responsive>
        <tbody>
          <tr>
            <td>Hash:</td>
            <td>
              <BlockLink id={block.id} />
            </td>
          </tr>
          <tr>
            <td>Epoch:</td>
            <td>
              <EpochLink number={block.date.epoch.id} />
            </td>
          </tr>
          <tr>
            <td>Slot:</td>
            <td>{block.date.slot}</td>
          </tr>
          <tr>
            <td>Chain length:</td>
            <td>
              <BlockLink chainLength={block.chainLength} />
            </td>
          </tr>
          <tr>
            <td>Previous block:</td>
            <td>
              <BlockLink id={block.previousBlock.id} />
            </td>
          </tr>
        </tbody>
      </Table>
    </div>
  </div>
);

export default createFragmentContainer(BlockInfo, {
  block: graphql`
    fragment BlockInfo_block on Block {
      id
      date {
        epoch {
          id
        }
        slot
      }
      chainLength
      previousBlock {
        id
      }
      transactions {
        ...TransactionTable_transactions
      }
    }
  `
});
