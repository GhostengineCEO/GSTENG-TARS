import React from 'react';
import styled from '@emotion/styled';
import Panel from '../common/Panel';
import Button from '../common/Button';

const Dashboard = styled(Panel)`
  display: flex;
  flex-direction: column;
  height: 100%;
`;

const Pose3D = styled.div`
  height: 200px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(255, 255, 255, 0.05);
  margin-bottom: 1rem;
`;

const SensorGrid = styled.div`
  flex: 1;
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 0.5rem;
`;

const SensorGraph = styled.div`
  background: rgba(255, 255, 255, 0.05);
  height: 100px;
  display: flex;
  align-items: center;
  justify-content: center;
`;

const Controls = styled.div`
  display: flex;
  justify-content: space-around;
  margin-top: 0.5rem;
`;

const Telemetry: React.FC = () => {
  return (
    <Dashboard>
      <Pose3D>3D Pose Placeholder</Pose3D>
      <SensorGrid>
        <SensorGraph>Sensor 1</SensorGraph>
        <SensorGraph>Sensor 2</SensorGraph>
        <SensorGraph>Sensor 3</SensorGraph>
        <SensorGraph>Sensor 4</SensorGraph>
      </SensorGrid>
      <Controls>
        <Button>Forward</Button>
        <Button>Stop</Button>
        <Button>Backward</Button>
      </Controls>
    </Dashboard>
  );
};

export default Telemetry;

