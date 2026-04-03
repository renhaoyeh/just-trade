import { BrowserRouter, Routes, Route } from "react-router";
import { AppLayout } from "@/components/layout/AppLayout";
import Dashboard from "@/pages/Dashboard";
import Sectors from "@/pages/Sectors";
import SectorDetail from "@/pages/SectorDetail";
import Analysis from "@/pages/Analysis";
import Settings from "@/pages/Settings";

function App() {
  return (
    <BrowserRouter>
      <Routes>
        <Route element={<AppLayout />}>
          <Route path="/" element={<Dashboard />} />
          <Route path="/sectors" element={<Sectors />} />
          <Route path="/sectors/:sectorName" element={<SectorDetail />} />
          <Route path="/trading-agents" element={<Analysis />} />
          <Route path="/settings" element={<Settings />} />
        </Route>
      </Routes>
    </BrowserRouter>
  );
}

export default App;
