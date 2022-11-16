import {
  Button,
  Flex,
  Text,
  IconButton,
  Switch,
  Divider,
  VStack,
  Kbd,
  Menu,
  MenuButton,
  MenuList,
  MenuItem
} from '@chakra-ui/react'
import { EditIcon, StarIcon } from '@chakra-ui/icons'
import { Keypress, Macro } from '../../types'
import { BaseSyntheticEvent } from 'react'
import { HIDLookup } from '../../maps/HIDmap'
import { updateBackendConfig } from '../../utils'
import { useApplicationContext } from '../../contexts/applicationContext'
import { ViewState } from '../../enums'

type Props = {
  macro: Macro
  index: number
  onDelete: (index: number) => void
}

function MacroCard({ macro, index, onDelete }: Props) {
  const { collections, changeSelectedMacroIndex, changeViewState } =
    useApplicationContext()

  const onToggle = (event: BaseSyntheticEvent) => {
    macro.active = event.target.checked
    // update backend
    updateBackendConfig(collections)
  }

  return (
    <VStack w="100%" border="1px" rounded="md" p="3" m="auto" spacing="8px">
      {/** Top Row */}
      <Flex w="100%" justifyContent="space-between">
        <Flex w="100%" gap="8px" alignItems="center">
          <StarIcon />
          <Text fontWeight="semibold">{macro.name}</Text>
        </Flex>
        <Menu>
          <MenuButton
            as={IconButton}
            aria-label="Kebab Menu Button"
            icon={
              <svg
                xmlns="http://www.w3.org/2000/svg"
                fill="none"
                viewBox="0 0 24 24"
                strokeWidth="1.5"
                stroke="black"
                width={24}
                height={24}
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  d="M12 6.75a.75.75 0 110-1.5.75.75 0 010 1.5zM12 12.75a.75.75 0 110-1.5.75.75 0 010 1.5zM12 18.75a.75.75 0 110-1.5.75.75 0 010 1.5z"
                />
              </svg>
            }
            variant="link"
          ></MenuButton>
          <MenuList>
            <MenuItem isDisabled>Duplicate</MenuItem>
            <MenuItem isDisabled>Move to Collection</MenuItem>
            <MenuItem isDisabled>Export</MenuItem>
            <Divider />
            <MenuItem onClick={() => onDelete(index)} textColor="red.500">
              Delete
            </MenuItem>
          </MenuList>
        </Menu>
      </Flex>
      {/** Trigger Keys Display */}
      <Text fontSize="sm" color="gray.600" alignSelf="self-start">
        Trigger Keys:
      </Text>
      <Flex w="100%" gap="4px">
        {macro.trigger.data.map((key: Keypress, index: number) => (
          <Kbd key={index}>{HIDLookup.get(key.keypress)?.displayString}</Kbd>
        ))}
      </Flex>
      <Divider />
      {/** Buttons */}
      <Flex w="100%" alignItems="center" justifyContent="space-between">
        <Button
          size="sm"
          leftIcon={<EditIcon />}
          onClick={() => {
            changeSelectedMacroIndex(index)
            changeViewState(ViewState.Editview)
          }}
        >
          Edit
        </Button>
        <Switch defaultChecked={macro.active} onChange={onToggle} />
      </Flex>
    </VStack>
  )
}

export default MacroCard
