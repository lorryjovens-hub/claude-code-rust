import type * as React from 'react'

export type WizardContextValue = {
  currentStep?: number
  totalSteps?: number
  next?: () => void
  back?: () => void
  goToStep?: (step: number) => void
}

export type WizardProviderProps = {
  children?: React.ReactNode
}
