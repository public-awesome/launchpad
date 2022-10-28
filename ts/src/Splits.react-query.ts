/**
* This file was automatically generated by @cosmwasm/ts-codegen@0.16.5.
* DO NOT MODIFY IT BY HAND. Instead, modify the source JSONSchema file,
* and run the @cosmwasm/ts-codegen generate command to regenerate this file.
*/

import { UseQueryOptions, useQuery, useMutation, UseMutationOptions } from "@tanstack/react-query";
import { ExecuteResult } from "@cosmjs/cosmwasm-stargate";
import { StdFee, Coin } from "@cosmjs/amino";
import { Cw4Contract, Addr, Config, ExecuteMsg, InstantiateMsg, QueryMsg } from "./Splits.types";
import { SplitsQueryClient, SplitsClient } from "./Splits.client";
export const splitsQueryKeys = {
  contract: ([{
    contract: "splits"
  }] as const),
  address: (contractAddress: string | undefined) => ([{ ...splitsQueryKeys.contract[0],
    address: contractAddress
  }] as const),
  config: (contractAddress: string | undefined, args?: Record<string, unknown>) => ([{ ...splitsQueryKeys.address(contractAddress)[0],
    method: "config",
    args
  }] as const),
  member: (contractAddress: string | undefined, args?: Record<string, unknown>) => ([{ ...splitsQueryKeys.address(contractAddress)[0],
    method: "member",
    args
  }] as const),
  listMembers: (contractAddress: string | undefined, args?: Record<string, unknown>) => ([{ ...splitsQueryKeys.address(contractAddress)[0],
    method: "list_members",
    args
  }] as const)
};
export interface SplitsReactQuery<TResponse, TData = TResponse> {
  client: SplitsQueryClient | undefined;
  options?: Omit<UseQueryOptions<TResponse, Error, TData>, "'queryKey' | 'queryFn' | 'initialData'"> & {
    initialData?: undefined;
  };
}
export interface SplitsListMembersQuery<TData> extends SplitsReactQuery<ListMembersResponse, TData> {
  args: {
    limit?: number;
    startAfter?: string;
  };
}
export function useSplitsListMembersQuery<TData = ListMembersResponse>({
  client,
  args,
  options
}: SplitsListMembersQuery<TData>) {
  return useQuery<ListMembersResponse, Error, TData>(splitsQueryKeys.listMembers(client?.contractAddress, args), () => client ? client.listMembers({
    limit: args.limit,
    startAfter: args.startAfter
  }) : Promise.reject(new Error("Invalid client")), { ...options,
    enabled: !!client && (options?.enabled != undefined ? options.enabled : true)
  });
}
export interface SplitsMemberQuery<TData> extends SplitsReactQuery<MemberResponse, TData> {
  args: {
    address: string;
  };
}
export function useSplitsMemberQuery<TData = MemberResponse>({
  client,
  args,
  options
}: SplitsMemberQuery<TData>) {
  return useQuery<MemberResponse, Error, TData>(splitsQueryKeys.member(client?.contractAddress, args), () => client ? client.member({
    address: args.address
  }) : Promise.reject(new Error("Invalid client")), { ...options,
    enabled: !!client && (options?.enabled != undefined ? options.enabled : true)
  });
}
export interface SplitsConfigQuery<TData> extends SplitsReactQuery<ConfigResponse, TData> {}
export function useSplitsConfigQuery<TData = ConfigResponse>({
  client,
  options
}: SplitsConfigQuery<TData>) {
  return useQuery<ConfigResponse, Error, TData>(splitsQueryKeys.config(client?.contractAddress), () => client ? client.config() : Promise.reject(new Error("Invalid client")), { ...options,
    enabled: !!client && (options?.enabled != undefined ? options.enabled : true)
  });
}
export interface SplitsDistributeMutation {
  client: SplitsClient;
  args?: {
    fee?: number | StdFee | "auto";
    memo?: string;
    funds?: Coin[];
  };
}
export function useSplitsDistributeMutation(options?: Omit<UseMutationOptions<ExecuteResult, Error, SplitsDistributeMutation>, "mutationFn">) {
  return useMutation<ExecuteResult, Error, SplitsDistributeMutation>(({
    client,
    args: {
      fee,
      memo,
      funds
    } = {}
  }) => client.distribute(fee, memo, funds), options);
}