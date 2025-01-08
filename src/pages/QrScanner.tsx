import { useLocation, useNavigate } from 'react-router-dom';
import { scan, Format, cancel } from '@tauri-apps/plugin-barcode-scanner';
import Layout from '@/components/Layout';
import Header from '@/components/Header';
import { useEffect, useCallback } from 'react';
import { useNavigationStore } from '@/state';

export default function QRScanner() {
  const navigate = useNavigate();
  const { state } = useLocation();
  const returnPath = state?.returnTo || '/';
  const { setReturnValue } = useNavigationStore();

  // Handle successful scan
  const handleScanSuccess = useCallback(
    (content: string) => {
      if (returnPath.startsWith('/offers')) {
        // For offers, navigate directly to the offer view
        navigate(`/offers/view/${encodeURIComponent(content.trim())}`, {
          replace: true,
        });
      } else {
        console.log('Scanned content:', content);
        setReturnValue(returnPath, { status: 'success', data: content });
        // For other components, maintain existing behavior
        navigate(-1);
        // navigate(returnPath, {
        //   replace: true,
        //   state: { scannedUri: content },
        // });
      }
    },
    [navigate, returnPath, setReturnValue],
  );

  const cancelScan = useCallback(() => {
    cancel()
      .catch(console.error)
      .finally(() => navigate(returnPath, { replace: true }));
  }, [navigate, returnPath]);

  useEffect(() => {
    const startScanning = async () => {
      try {
        const result = await scan({
          windowed: true,
          formats: [Format.QRCode],
        });

        if (result) {
          await cancel().catch(console.error);
          handleScanSuccess(result.content);
        }
      } catch (error) {
        console.error('Scan failed:', error);
        navigate(returnPath, { replace: true });
      }
    };

    startScanning();

    return () => {
      cancel().catch(console.error);
    };
  }, [navigate, handleScanSuccess]);

  return (
    <Layout transparentBackground={true}>
      <Header title='Scan QR Code' back={cancelScan} />
      <div className='relative w-full h-full bg-transparent'>
        <div className='absolute inset-0 bg-black bg-opacity-0'>
          <div className='absolute top-1/2 left-1/2 transform -translate-x-1/2 -translate-y-1/2'>
            <div className='relative w-64 h-64'>
              <div className='absolute top-0 left-0 w-8 h-8 border-l-4 border-t-4 border-white' />
              <div className='absolute top-0 right-0 w-8 h-8 border-r-4 border-t-4 border-white' />
              <div className='absolute bottom-0 left-0 w-8 h-8 border-l-4 border-b-4 border-white' />
              <div className='absolute bottom-0 right-0 w-8 h-8 border-r-4 border-b-4 border-white' />
            </div>
            <p className='text-white text-center mt-8'>
              Position the QR code within the frame
            </p>
          </div>
        </div>
      </div>
    </Layout>
  );
}
