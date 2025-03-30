import { formatTimestamp, fromMojos } from '@/lib/utils';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { ColumnDef, Row, RowSelectionState } from '@tanstack/react-table';
import { openUrl } from '@tauri-apps/plugin-opener';
import { ArrowDown, ArrowUp, FilterIcon, FilterXIcon } from 'lucide-react';
import { CoinRecord, CoinSortMode } from '../bindings';
import { NumberFormat } from './NumberFormat';
import { SimplePagination } from './SimplePagination';
import { Button } from './ui/button';
import { Checkbox } from './ui/checkbox';
import { DataTable } from './ui/data-table';

export interface CoinListProps {
  precision: number;
  coins: CoinRecord[];
  selectedCoins: RowSelectionState;
  setSelectedCoins: React.Dispatch<React.SetStateAction<RowSelectionState>>;
  currentPage: number;
  totalPages: number;
  setCurrentPage: (page: number) => void;
  maxRows: number;
  actions?: React.ReactNode;
  // Backend sorting and filtering props
  sortMode: CoinSortMode;
  sortDirection: boolean; // true = ascending, false = descending
  includeSpentCoins: boolean;
  onSortModeChange: (mode: CoinSortMode) => void;
  onSortDirectionChange: (ascending: boolean) => void;
  onIncludeSpentCoinsChange: (include: boolean) => void;
}

export default function CoinList(props: CoinListProps) {
  // Column definitions
  const columns: ColumnDef<CoinRecord>[] = [
    {
      id: 'select',
      size: 30,
      header: ({ table }) => (
        <div className='flex justify-center items-center'>
          <Checkbox
            checked={
              table.getIsAllPageRowsSelected() ||
              (table.getIsSomePageRowsSelected() && 'indeterminate')
            }
            onCheckedChange={(value) => {
              table.toggleAllPageRowsSelected(!!value);

              // Update the external selection state
              const newSelections = { ...props.selectedCoins };

              // Get all visible rows on the current page
              const pageRows = table.getRowModel().rows;

              pageRows.forEach((row) => {
                const rowId = row.original.coin_id;
                newSelections[rowId] = !!value;
              });

              props.setSelectedCoins(newSelections);
            }}
            aria-label={t`Select all coins`}
          />
        </div>
      ),
      cell: ({ row }) => (
        <div className='flex justify-center items-center'>
          <Checkbox
            checked={row.getIsSelected()}
            onCheckedChange={(value) => {
              row.toggleSelected(!!value);

              // Update the external selection state
              const rowId = row.original.coin_id;
              props.setSelectedCoins((prev) => ({
                ...prev,
                [rowId]: !!value,
              }));
            }}
            aria-label={t`Select coin row`}
          />
        </div>
      ),
      enableSorting: false,
    },
    {
      accessorKey: 'coin_id',
      size: 100,
      header: () => (
        <Button
          className='px-0'
          variant='link'
          onClick={() => {
            if (props.sortMode === 'parent_coin_id') {
              // Toggle direction only if already sorting by this column
              props.onSortDirectionChange(!props.sortDirection);
            } else {
              // Set column as sort field with default direction (descending)
              props.onSortModeChange('parent_coin_id');
              props.onSortDirectionChange(false);
            }
            props.setCurrentPage(0);
          }}
        >
          <Trans>Coin ID</Trans>
          {props.sortMode === 'parent_coin_id' ? (
            props.sortDirection ? (
              <ArrowUp className='ml-2 h-4 w-4' aria-hidden='true' />
            ) : (
              <ArrowDown className='ml-2 h-4 w-4' aria-hidden='true' />
            )
          ) : (
            <span className='ml-2 w-4 h-4' />
          )}
        </Button>
      ),
      cell: ({ row }) => {
        const coinId = row.original.coin_id;
        return (
          <div
            className='cursor-pointer truncate hover:underline'
            onClick={(e) => {
              e.stopPropagation();
              openUrl(`https://spacescan.io/coin/0x${coinId}`);
            }}
            aria-label={t`View coin ${coinId} on Spacescan.io`}
            role='button'
            onKeyDown={(e) => {
              if (e.key === 'Enter' || e.key === ' ') {
                e.stopPropagation();
                openUrl(`https://spacescan.io/coin/0x${coinId}`);
              }
            }}
          >
            {coinId}
          </div>
        );
      },
    },
    {
      accessorKey: 'amount',
      size: 100,
      header: () => (
        <Button
          className='px-0'
          variant='link'
          onClick={() => {
            if (props.sortMode === 'amount') {
              // Toggle direction only if already sorting by this column
              props.onSortDirectionChange(!props.sortDirection);
            } else {
              // Set column as sort field with default direction (descending)
              props.onSortModeChange('amount');
              props.onSortDirectionChange(false);
            }
            props.setCurrentPage(0);
          }}
        >
          <span className='text-foreground hover:underline'>
            <Trans>Amount</Trans>
          </span>
          {props.sortMode === 'amount' ? (
            props.sortDirection ? (
              <ArrowUp className='ml-2 h-4 w-4' aria-hidden='true' />
            ) : (
              <ArrowDown className='ml-2 h-4 w-4' aria-hidden='true' />
            )
          ) : (
            <span className='ml-2 w-4 h-4' />
          )}
        </Button>
      ),
      cell: (info) => (
        <div className='font-mono truncate'>
          <NumberFormat
            value={fromMojos(info.getValue() as string, props.precision)}
            minimumFractionDigits={0}
            maximumFractionDigits={props.precision}
          />
        </div>
      ),
    },
    {
      accessorKey: 'created_height',
      header: () => (
        <Button
          className='px-0'
          variant='link'
          onClick={() => {
            if (props.sortMode === 'created_height') {
              // Toggle direction only if already sorting by this column
              props.onSortDirectionChange(!props.sortDirection);
            } else {
              // Set column as sort field with default direction (descending)
              props.onSortModeChange('created_height');
              props.onSortDirectionChange(false);
            }
            props.setCurrentPage(0);
          }}
        >
          <Trans>Confirmed</Trans>
          {props.sortMode === 'created_height' ? (
            props.sortDirection ? (
              <ArrowUp className='ml-2 h-4 w-4' aria-hidden='true' />
            ) : (
              <ArrowDown className='ml-2 h-4 w-4' aria-hidden='true' />
            )
          ) : (
            <span className='ml-2 w-4 h-4' />
          )}
        </Button>
      ),
      size: 140,
      cell: ({ row }) =>
        row.original.created_timestamp
          ? formatTimestamp(row.original.created_timestamp, 'short', 'short')
          : row.original.created_height
            ? row.original.created_height.toString()
            : row.original.create_transaction_id
              ? t`Pending...`
              : '',
    },
    {
      accessorKey: 'spent_height',
      header: () => (
        <div className='flex items-center space-x-1'>
          <Button
            className='px-0'
            variant='link'
            onClick={() => {
              if (props.sortMode === 'spent_height') {
                // Toggle direction only if already sorting by this column
                props.onSortDirectionChange(!props.sortDirection);
              } else {
                // Set column as sort field with default direction (descending)
                props.onSortModeChange('spent_height');
                props.onSortDirectionChange(false);
              }
              props.setCurrentPage(0);
            }}
          >
            <Trans>Spent</Trans>
            {props.sortMode === 'spent_height' ? (
              props.sortDirection ? (
                <ArrowUp className='ml-2 h-4 w-4' aria-hidden='true' />
              ) : (
                <ArrowDown className='ml-2 h-4 w-4' aria-hidden='true' />
              )
            ) : (
              <span className='ml-2 w-4 h-4' />
            )}
          </Button>
          <Button
            variant='ghost'
            size='icon'
            className='ml-2 h-4 w-4'
            onClick={() => {
              const newIncludeSpentCoins = !props.includeSpentCoins;
              props.onIncludeSpentCoinsChange(newIncludeSpentCoins);
              if (newIncludeSpentCoins) {
                props.onSortModeChange('spent_height');
                props.onSortDirectionChange(false);
              } else {
                props.onSortModeChange('created_height');
                props.onSortDirectionChange(false);
              }
              props.setCurrentPage(0);
            }}
            aria-label={
              props.includeSpentCoins
                ? t`Hide spent coins`
                : t`Show spent coins`
            }
          >
            {props.includeSpentCoins ? (
              <FilterXIcon className='h-3 w-3' aria-hidden='true' />
            ) : (
              <FilterIcon className='h-3 w-3' aria-hidden='true' />
            )}
          </Button>
        </div>
      ),
      size: 120,
      cell: ({ row }) =>
        row.original.spent_timestamp
          ? formatTimestamp(row.original.spent_timestamp, 'short', 'short')
          : row.original.spent_height
            ? row.original.spent_height.toString()
            : row.original.spend_transaction_id
              ? t`Pending...`
              : row.original.offer_id
                ? t`Locked in offer`
                : '',
    },
  ];

  const getRowStyles = (row: Row<CoinRecord>) => {
    const coin = row.original;
    const isSpent = !!coin.spent_height || !!coin.spend_transaction_id;
    const isPending = !coin.created_height;

    let className = '';

    if (isSpent) {
      className = 'opacity-50 relative';
    } else if (isPending) {
      className = 'font-medium';
    }

    return {
      className,
      onClick: () => {
        const newValue = !row.getIsSelected();
        row.toggleSelected(newValue);

        // Update the external selection state
        const rowId = row.original.coin_id;
        props.setSelectedCoins((prev) => ({
          ...prev,
          [rowId]: newValue,
        }));
      },
    };
  };

  return (
    <div>
      <DataTable
        data={props.coins}
        columns={columns}
        getRowStyles={getRowStyles}
        state={{
          rowSelection: props.selectedCoins,
          maxRows: props.maxRows,
        }}
      />
      <div className='flex-shrink-0 py-4'>
        <SimplePagination
          currentPage={props.currentPage}
          pageCount={props.totalPages}
          setCurrentPage={props.setCurrentPage}
          size='sm'
          align='between'
          actions={props.actions}
        />
      </div>
    </div>
  );
}
