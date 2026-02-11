import { useEffect } from "react";
import { useIdentityStore } from "./stores/identityStore";
import SetupScreen from "./components/identity/SetupScreen";
import AppLayout from "./components/layout/AppLayout";

function App() {
  const { isLoading, isSetup, loadIdentity } = useIdentityStore();

  useEffect(() => {
    loadIdentity();
  }, [loadIdentity]);

  if (isLoading) {
    return (
      <div className="h-full bg-gray-900 flex items-center justify-center">
        <div className="text-center">
          <div className="w-12 h-12 border-4 border-indigo-500 border-t-transparent rounded-full animate-spin mx-auto mb-4" />
          <p className="text-gray-400">Starting Chatr...</p>
        </div>
      </div>
    );
  }

  if (!isSetup) {
    return <SetupScreen />;
  }

  return <AppLayout />;
}

export default App;
