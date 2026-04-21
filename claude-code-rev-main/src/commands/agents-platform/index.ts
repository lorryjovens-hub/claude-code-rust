const agentsPlatform = {
  name: 'agents-platform',
  type: 'local',
  description:
    'Reserved internal command. This restored build keeps the command visible but does not include the original agents platform backend.',
  supportsNonInteractive: true,
  load: async () => ({
    async call() {
      return {
        type: 'text' as const,
        value:
          'agents-platform is not included in this restored workspace.\n\n' +
          'The command shell is present so callers fail cleanly, but the ' +
          'internal backend that powers platform-managed agents was not ' +
          'recoverable from source maps.',
      }
    },
  }),
}

export default agentsPlatform
